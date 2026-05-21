// rust-style-allow: large-file because the JSON-RPC dispatch loop, server
// state, tool-result builders, and host-result projections for `runx mcp
// serve` all sit on the same protocol surface.
use std::io::{Read, Write};
#[cfg(feature = "mcp-rmcp")]
use std::pin::Pin;
#[cfg(feature = "mcp-rmcp")]
use std::sync::{Arc, Mutex};
#[cfg(feature = "mcp-rmcp")]
use std::task::{Context, Poll};

use runx_contracts::{JsonObject, JsonValue};

#[cfg(all(feature = "mcp", not(feature = "mcp-rmcp")))]
use super::framing::{content_length, find_header_end};
#[cfg(all(feature = "mcp", not(feature = "mcp-rmcp")))]
use super::jsonrpc::{PROTOCOL_VERSION, json_rpc_error, json_rpc_response};
#[cfg(feature = "mcp-rmcp")]
use super::rmcp_content_length::{RmcpContentLengthTransport, RmcpTransportErrorState};
use super::server_skill::{execute_mcp_server_skill, identifier_segment};
use super::types::{
    McpContent, McpHostRunResult, McpServerError, McpServerOptions, McpServerTool,
    McpServerToolBehavior, McpToolResult,
};

const MAX_SERVER_REQUEST_BYTES: usize = 4 * 1024 * 1024;

pub fn serve_mcp_json_rpc(
    input: impl Read,
    output: impl Write,
    options: McpServerOptions,
) -> Result<(), McpServerError> {
    assert_unique_server_tool_names(&options.tools)?;
    #[cfg(feature = "mcp-rmcp")]
    {
        serve_mcp_json_rpc_with_rmcp(input, output, options)
    }
    #[cfg(not(feature = "mcp-rmcp"))]
    {
        serve_mcp_json_rpc_checked(input, output, options)
    }
}

pub fn mcp_tool_result_from_host_result(result: McpHostRunResult) -> McpToolResult {
    match result {
        McpHostRunResult::Completed {
            skill_name,
            output,
            receipt_id,
            runx,
        } => completed_mcp_tool_result(skill_name, output, receipt_id, runx),
        McpHostRunResult::NeedsAgent {
            skill_name,
            run_id,
            request_count,
            runx,
        } => needs_agent_mcp_tool_result(skill_name, run_id, request_count, runx),
        McpHostRunResult::Denied {
            skill_name,
            receipt_id,
            runx,
        } => denied_mcp_tool_result(skill_name, receipt_id, runx),
        McpHostRunResult::Escalated {
            skill_name,
            receipt_id,
            error,
            runx,
        } => escalated_mcp_tool_result(skill_name, receipt_id, error, runx),
        McpHostRunResult::Failed {
            skill_name,
            receipt_id,
            error,
            runx,
        } => failed_mcp_tool_result(skill_name, receipt_id, error, runx),
    }
}

fn completed_mcp_tool_result(
    skill_name: String,
    output: String,
    receipt_id: String,
    runx: JsonObject,
) -> McpToolResult {
    let text = if output.trim().is_empty() {
        format!("{skill_name} completed. Inspect receipt {receipt_id}.")
    } else {
        output
    };
    mcp_host_tool_result(text, runx, false)
}

fn needs_agent_mcp_tool_result(
    skill_name: String,
    run_id: String,
    request_count: usize,
    runx: JsonObject,
) -> McpToolResult {
    mcp_host_tool_result(
        format!(
            "{skill_name} needs agent input at {run_id}. Continue by rerunning the same skill with --run-id {run_id} --answers answers.json after resolving {request_count} request(s)."
        ),
        runx,
        false,
    )
}

fn denied_mcp_tool_result(
    skill_name: String,
    receipt_id: Option<String>,
    runx: JsonObject,
) -> McpToolResult {
    let text = match receipt_id {
        Some(receipt_id) => format!("{skill_name} was denied by policy (receipt {receipt_id})."),
        None => format!("{skill_name} was denied by policy."),
    };
    mcp_host_tool_result(text, runx, true)
}

fn escalated_mcp_tool_result(
    skill_name: String,
    receipt_id: String,
    error: String,
    runx: JsonObject,
) -> McpToolResult {
    mcp_host_tool_result(
        format!("{skill_name} escalated. Inspect receipt {receipt_id}. {error}")
            .trim()
            .to_owned(),
        runx,
        true,
    )
}

fn failed_mcp_tool_result(
    skill_name: String,
    receipt_id: Option<String>,
    error: String,
    runx: JsonObject,
) -> McpToolResult {
    mcp_host_tool_result(
        format!(
            "{skill_name} failed. Inspect receipt {}. {error}",
            receipt_id.unwrap_or_else(|| "n/a".to_owned())
        )
        .trim()
        .to_owned(),
        runx,
        true,
    )
}

fn mcp_host_tool_result(text: String, runx: JsonObject, is_error: bool) -> McpToolResult {
    McpToolResult {
        content: vec![McpContent { text }],
        structured_content: Some(runx_content(runx)),
        is_error,
    }
}

fn runx_content(runx: JsonObject) -> JsonObject {
    [("runx".to_owned(), JsonValue::Object(runx))].into()
}

#[cfg(all(feature = "mcp", not(feature = "mcp-rmcp")))]
fn serve_mcp_json_rpc_checked(
    mut input: impl Read,
    mut output: impl Write,
    options: McpServerOptions,
) -> Result<(), McpServerError> {
    let mut state = McpServerState::new(options);
    let mut buffer = Vec::new();
    let mut chunk = [0_u8; 8192];
    loop {
        let read = input
            .read(&mut chunk)
            .map_err(|error| McpServerError::new(format!("MCP request read failed: {error}")))?;
        if read == 0 {
            return Ok(());
        }
        buffer.extend_from_slice(&chunk[..read]);
        if buffer.len() > MAX_SERVER_REQUEST_BYTES {
            return Err(McpServerError::new(format!(
                "MCP request exceeded {MAX_SERVER_REQUEST_BYTES}-byte size limit."
            )));
        }
        write_available_server_responses(&mut buffer, &mut output, &mut state)?;
    }
}

#[cfg(feature = "mcp-rmcp")]
fn serve_mcp_json_rpc_with_rmcp(
    mut input: impl Read,
    mut output: impl Write,
    options: McpServerOptions,
) -> Result<(), McpServerError> {
    let mut input_bytes = Vec::new();
    input
        .read_to_end(&mut input_bytes)
        .map_err(|error| McpServerError::new(format!("MCP request read failed: {error}")))?;
    if input_bytes.len() > MAX_SERVER_REQUEST_BYTES {
        return Err(McpServerError::new(format!(
            "MCP request exceeded {MAX_SERVER_REQUEST_BYTES}-byte size limit."
        )));
    }
    let output_bytes = block_on_rmcp_server(input_bytes, options)?;
    output
        .write_all(&output_bytes)
        .and_then(|()| output.flush())
        .map_err(|error| McpServerError::new(format!("MCP response write failed: {error}")))
}

#[cfg(feature = "mcp-rmcp")]
fn block_on_rmcp_server(
    input: Vec<u8>,
    options: McpServerOptions,
) -> Result<Vec<u8>, McpServerError> {
    tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
        .map_err(|error| {
            McpServerError::new(format!("MCP server runtime initialization failed: {error}"))
        })?
        .block_on(serve_mcp_json_rpc_with_rmcp_async(input, options))
}

#[cfg(feature = "mcp-rmcp")]
async fn serve_mcp_json_rpc_with_rmcp_async(
    input: Vec<u8>,
    options: McpServerOptions,
) -> Result<Vec<u8>, McpServerError> {
    let (mut client_write, server_read) = tokio::io::duplex(input.len().max(1));
    tokio::io::AsyncWriteExt::write_all(&mut client_write, &input)
        .await
        .map_err(|error| McpServerError::new(format!("MCP request write failed: {error}")))?;
    tokio::io::AsyncWriteExt::shutdown(&mut client_write)
        .await
        .map_err(|error| McpServerError::new(format!("MCP request shutdown failed: {error}")))?;

    let output = SharedAsyncOutput::default();
    let output_bytes = output.bytes();
    let error_state = RmcpTransportErrorState::default();
    let transport = RmcpContentLengthTransport::new(
        server_read,
        output,
        MAX_SERVER_REQUEST_BYTES,
        error_state.clone(),
    );
    let service = RmcpProofServer {
        state: Mutex::new(McpServerState::new(options)),
    };
    let running = rmcp::serve_server(service, transport)
        .await
        .map_err(|error| {
            McpServerError::new(format!(
                "MCP rmcp server initialization failed: {}",
                error_state.take().unwrap_or_else(|| error.to_string())
            ))
        })?;
    running
        .waiting()
        .await
        .map_err(|error| McpServerError::new(format!("MCP rmcp server task failed: {error}")))?;
    output_bytes
        .lock()
        .map(|bytes| bytes.clone())
        .map_err(|_| McpServerError::new("MCP rmcp server output lock failed."))
}

#[cfg(feature = "mcp-rmcp")]
struct RmcpProofServer {
    state: Mutex<McpServerState>,
}

#[cfg(feature = "mcp-rmcp")]
impl rmcp::ServerHandler for RmcpProofServer {
    fn get_info(&self) -> rmcp::model::ServerInfo {
        let (package_name, package_version) = self.state.lock().map_or_else(
            |_| ("runx-mcp".to_owned(), "0.0.0".to_owned()),
            |state| {
                (
                    state.options.package_name.clone(),
                    state.options.package_version.clone(),
                )
            },
        );
        rmcp::model::ServerInfo::new(
            rmcp::model::ServerCapabilities::builder()
                .enable_tools()
                .build(),
        )
        .with_protocol_version(rmcp::model::ProtocolVersion::V_2025_06_18)
        .with_server_info(rmcp::model::Implementation::new(
            package_name,
            package_version,
        ))
    }

    fn list_tools(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> impl Future<Output = Result<rmcp::model::ListToolsResult, rmcp::ErrorData>> + Send + '_
    {
        let result = self
            .state
            .lock()
            .map_err(|_| rmcp_internal_error("MCP server state lock failed."))
            .map(|state| {
                rmcp::model::ListToolsResult::with_all_items(
                    state
                        .options
                        .tools
                        .iter()
                        .map(rmcp_tool_from_server_tool)
                        .collect(),
                )
            });
        std::future::ready(result)
    }

    fn call_tool(
        &self,
        request: rmcp::model::CallToolRequestParams,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> impl Future<Output = Result<rmcp::model::CallToolResult, rmcp::ErrorData>> + Send + '_
    {
        let result = self
            .state
            .lock()
            .map_err(|_| rmcp_internal_error("MCP server state lock failed."))
            .and_then(|mut state| {
                let arguments = match request.arguments {
                    Some(arguments) => runx_json_object(arguments).map_err(rmcp_invalid_params)?,
                    None => JsonObject::new(),
                };
                handle_rmcp_tool_call(&mut state, &request.name, arguments)
            });
        std::future::ready(result)
    }

    fn get_tool(&self, name: &str) -> Option<rmcp::model::Tool> {
        self.state
            .lock()
            .ok()
            .and_then(|state| {
                state
                    .options
                    .tools
                    .iter()
                    .find(|tool| tool.name == name)
                    .cloned()
            })
            .map(|tool| rmcp_tool_from_server_tool(&tool))
    }
}

#[cfg(feature = "mcp-rmcp")]
fn handle_rmcp_tool_call(
    state: &mut McpServerState,
    name: &str,
    arguments: JsonObject,
) -> Result<rmcp::model::CallToolResult, rmcp::ErrorData> {
    let Some(tool) = state.options.tools.iter().find(|tool| tool.name == name) else {
        return Err(rmcp::ErrorData::new(
            rmcp::model::ErrorCode::METHOD_NOT_FOUND,
            format!("tool not found: {name}"),
            None,
        ));
    };
    match tool.result.clone() {
        McpServerToolBehavior::Fixed(result) => rmcp_call_tool_result(result),
        McpServerToolBehavior::Skill(execution) => {
            match execute_mcp_server_skill(state, *execution, arguments) {
                Ok(result) => rmcp_call_tool_result(result),
                Err(error) => Err(rmcp_internal_error(error.to_string())),
            }
        }
    }
}

#[cfg(feature = "mcp-rmcp")]
fn rmcp_tool_from_server_tool(tool: &McpServerTool) -> rmcp::model::Tool {
    rmcp::model::Tool::new(
        tool.name.clone(),
        tool.description.clone(),
        Arc::new(rmcp_json_object(JsonValue::Object(
            tool.input_schema.clone(),
        ))),
    )
}

#[cfg(feature = "mcp-rmcp")]
fn rmcp_call_tool_result(
    result: McpToolResult,
) -> Result<rmcp::model::CallToolResult, rmcp::ErrorData> {
    let content = result
        .content
        .into_iter()
        .map(|entry| rmcp::model::Content::text(entry.text))
        .collect();
    let mut call_result = if result.is_error {
        rmcp::model::CallToolResult::error(content)
    } else {
        rmcp::model::CallToolResult::success(content)
    };
    call_result.structured_content = result
        .structured_content
        .map(|content| serde_json::to_value(content).map_err(rmcp_internal_error))
        .transpose()?;
    Ok(call_result)
}

#[cfg(feature = "mcp-rmcp")]
fn rmcp_json_object(value: JsonValue) -> rmcp::model::JsonObject {
    serde_json::to_vec(&value)
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default()
}

#[cfg(feature = "mcp-rmcp")]
fn runx_json_object(value: rmcp::model::JsonObject) -> Result<JsonObject, serde_json::Error> {
    serde_json::to_vec(&value).and_then(|bytes| serde_json::from_slice(&bytes))
}

#[cfg(feature = "mcp-rmcp")]
fn rmcp_invalid_params(error: impl std::fmt::Display) -> rmcp::ErrorData {
    rmcp::ErrorData::invalid_params(error.to_string(), None)
}

#[cfg(feature = "mcp-rmcp")]
fn rmcp_internal_error(error: impl std::fmt::Display) -> rmcp::ErrorData {
    rmcp::ErrorData::internal_error(error.to_string(), None)
}

#[cfg(feature = "mcp-rmcp")]
#[derive(Clone, Default)]
struct SharedAsyncOutput {
    bytes: Arc<Mutex<Vec<u8>>>,
}

#[cfg(feature = "mcp-rmcp")]
impl SharedAsyncOutput {
    fn bytes(&self) -> Arc<Mutex<Vec<u8>>> {
        Arc::clone(&self.bytes)
    }
}

#[cfg(feature = "mcp-rmcp")]
impl tokio::io::AsyncWrite for SharedAsyncOutput {
    fn poll_write(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        let mut bytes = self
            .bytes
            .lock()
            .map_err(|_| std::io::Error::other("MCP output lock failed."))?;
        bytes.extend_from_slice(buf);
        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Poll::Ready(Ok(()))
    }
}

#[derive(Debug)]
pub(super) struct McpServerState {
    options: McpServerOptions,
    next_run_sequence: u64,
}

impl McpServerState {
    fn new(options: McpServerOptions) -> Self {
        Self {
            options,
            next_run_sequence: 0,
        }
    }

    pub(super) fn next_run_id(&mut self, skill_name: &str) -> String {
        self.next_run_sequence = self.next_run_sequence.saturating_add(1);
        format!(
            "rx_mcp_{}_{}",
            identifier_segment(skill_name),
            self.next_run_sequence
        )
    }
}

#[cfg(all(feature = "mcp", not(feature = "mcp-rmcp")))]
fn write_available_server_responses(
    buffer: &mut Vec<u8>,
    output: &mut impl Write,
    state: &mut McpServerState,
) -> Result<(), McpServerError> {
    while let Some(header_end) = find_header_end(buffer) {
        let header = std::str::from_utf8(&buffer[..header_end])
            .map_err(|_| McpServerError::new("MCP request header must be UTF-8."))?;
        let Some(content_length) = content_length(header) else {
            return Err(McpServerError::new(
                "MCP request requires a Content-Length header.",
            ));
        };
        if content_length > MAX_SERVER_REQUEST_BYTES {
            return Err(McpServerError::new(format!(
                "MCP request declared Content-Length {content_length}, exceeding {MAX_SERVER_REQUEST_BYTES}-byte limit."
            )));
        }
        let body_start = header_end + 4;
        let body_end = body_start.saturating_add(content_length);
        if buffer.len() < body_end {
            return Ok(());
        }
        let body = buffer[body_start..body_end].to_vec();
        buffer.drain(..body_end);
        let response = match serde_json::from_slice::<JsonValue>(&body) {
            Ok(request) => handle_mcp_server_request(state, request),
            Err(_) => Some(json_rpc_error(JsonValue::Null, -32700, "parse error")),
        };
        if let Some(response) = response {
            write_framed_json(output, &response)?;
        }
    }
    Ok(())
}

#[cfg(all(feature = "mcp", not(feature = "mcp-rmcp")))]
fn handle_mcp_server_request(state: &mut McpServerState, request: JsonValue) -> Option<JsonValue> {
    let JsonValue::Object(record) = request else {
        return Some(json_rpc_error(JsonValue::Null, -32600, "invalid request"));
    };
    let id = record.get("id").cloned().unwrap_or(JsonValue::Null);
    let method = match record.get("method") {
        Some(JsonValue::String(method)) => method.as_str(),
        _ => return Some(json_rpc_error(id, -32600, "invalid request")),
    };
    match method {
        "initialize" => Some(json_rpc_response(
            id,
            initialize_server_result(&state.options),
        )),
        "ping" => Some(json_rpc_response(id, JsonValue::Object(JsonObject::new()))),
        "tools/list" => Some(json_rpc_response(
            id,
            tools_list_result(&state.options.tools),
        )),
        "tools/call" => Some(handle_mcp_server_tool_call(state, id, record.get("params"))),
        _ if matches!(record.get("id"), None | Some(JsonValue::Null)) => None,
        _ => Some(json_rpc_error(id, -32601, "method not found")),
    }
}

#[cfg(all(feature = "mcp", not(feature = "mcp-rmcp")))]
fn handle_mcp_server_tool_call(
    state: &mut McpServerState,
    id: JsonValue,
    params: Option<&JsonValue>,
) -> JsonValue {
    let Some(JsonValue::Object(params)) = params else {
        return json_rpc_error(id, -32602, "invalid tool call");
    };
    let Some(JsonValue::String(name)) = params.get("name") else {
        return json_rpc_error(id, -32602, "invalid tool call");
    };
    if let Some(arguments) = params.get("arguments")
        && !matches!(arguments, JsonValue::Object(_))
    {
        return json_rpc_error(id, -32602, "tool arguments must be an object");
    }
    let Some(tool) = state.options.tools.iter().find(|tool| &tool.name == name) else {
        return json_rpc_error(id, -32601, &format!("tool not found: {name}"));
    };
    let arguments = match params.get("arguments") {
        Some(JsonValue::Object(arguments)) => arguments.clone(),
        _ => JsonObject::new(),
    };
    match tool.result.clone() {
        McpServerToolBehavior::Fixed(result) => {
            json_rpc_response(id, mcp_tool_result_json(&result))
        }
        McpServerToolBehavior::Skill(execution) => {
            match execute_mcp_server_skill(state, *execution, arguments) {
                Ok(result) => json_rpc_response(id, mcp_tool_result_json(&result)),
                Err(error) => json_rpc_error(id, -32000, &error.to_string()),
            }
        }
    }
}

#[cfg(all(feature = "mcp", not(feature = "mcp-rmcp")))]
fn initialize_server_result(options: &McpServerOptions) -> JsonValue {
    JsonValue::Object(
        [
            (
                "protocolVersion".to_owned(),
                JsonValue::String(PROTOCOL_VERSION.to_owned()),
            ),
            (
                "capabilities".to_owned(),
                JsonValue::Object(
                    [("tools".to_owned(), JsonValue::Object(JsonObject::new()))].into(),
                ),
            ),
            (
                "serverInfo".to_owned(),
                JsonValue::Object(
                    [
                        (
                            "name".to_owned(),
                            JsonValue::String(options.package_name.clone()),
                        ),
                        (
                            "version".to_owned(),
                            JsonValue::String(options.package_version.clone()),
                        ),
                    ]
                    .into(),
                ),
            ),
        ]
        .into(),
    )
}

#[cfg(all(feature = "mcp", not(feature = "mcp-rmcp")))]
fn tools_list_result(tools: &[McpServerTool]) -> JsonValue {
    JsonValue::Object(
        [(
            "tools".to_owned(),
            JsonValue::Array(tools.iter().map(server_tool_json).collect()),
        )]
        .into(),
    )
}

#[cfg(all(feature = "mcp", not(feature = "mcp-rmcp")))]
fn server_tool_json(tool: &McpServerTool) -> JsonValue {
    JsonValue::Object(
        [
            ("name".to_owned(), JsonValue::String(tool.name.clone())),
            (
                "description".to_owned(),
                JsonValue::String(tool.description.clone()),
            ),
            (
                "inputSchema".to_owned(),
                JsonValue::Object(tool.input_schema.clone()),
            ),
        ]
        .into(),
    )
}

#[cfg(all(feature = "mcp", not(feature = "mcp-rmcp")))]
fn mcp_tool_result_json(result: &McpToolResult) -> JsonValue {
    let mut record = JsonObject::new();
    record.insert(
        "content".to_owned(),
        JsonValue::Array(
            result
                .content
                .iter()
                .map(|entry| {
                    JsonValue::Object(
                        [
                            ("type".to_owned(), JsonValue::String("text".to_owned())),
                            ("text".to_owned(), JsonValue::String(entry.text.clone())),
                        ]
                        .into(),
                    )
                })
                .collect(),
        ),
    );
    if let Some(structured_content) = &result.structured_content {
        record.insert(
            "structuredContent".to_owned(),
            JsonValue::Object(structured_content.clone()),
        );
    }
    if result.is_error {
        record.insert("isError".to_owned(), JsonValue::Bool(true));
    }
    JsonValue::Object(record)
}

#[cfg(all(feature = "mcp", not(feature = "mcp-rmcp")))]
fn write_framed_json(output: &mut impl Write, message: &JsonValue) -> Result<(), McpServerError> {
    let body = serde_json::to_vec(message).map_err(|error| {
        McpServerError::new(format!("MCP response serialization failed: {error}"))
    })?;
    write!(output, "Content-Length: {}\r\n\r\n", body.len())
        .and_then(|()| output.write_all(&body))
        .and_then(|()| output.flush())
        .map_err(|error| McpServerError::new(format!("MCP response write failed: {error}")))
}

fn assert_unique_server_tool_names(tools: &[McpServerTool]) -> Result<(), McpServerError> {
    let mut seen = std::collections::BTreeSet::new();
    for tool in tools {
        if !seen.insert(tool.name.as_str()) {
            return Err(McpServerError::new(format!(
                "runx mcp serve received duplicate tool name '{}'. Serve unique skill names only.",
                tool.name
            )));
        }
    }
    Ok(())
}
