// rust-style-allow: large-file because the process supervisor, contract
// validation, timeout handling, and frame normalization must stay adjacent to
// keep the external adapter boundary auditable.
use std::collections::BTreeMap;
use std::io::{Read, Write};
#[cfg(unix)]
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use runx_contracts::{
    EXTERNAL_ADAPTER_PROTOCOL_VERSION, ExternalAdapterCancellationFrame,
    ExternalAdapterCredentialRequest, ExternalAdapterInvocation, ExternalAdapterManifest,
    ExternalAdapterResponse, ExternalAdapterStatus, ExternalAdapterTransportKind, JsonNumber,
    JsonObject, JsonValue, Reference, ReferenceType,
};
use thiserror::Error;

use crate::RuntimeError;
use crate::adapter::{InvocationStatus, SkillAdapter, SkillInvocation, SkillOutput};
use crate::receipts::paths::RUNX_RECEIPT_DIR_ENV;
use crate::time::now_iso8601;

const MANIFEST_INLINE_FIELD: &str = "external_adapter_manifest";
const MANIFEST_NESTED_FIELD: &str = "external_adapter";
const MANIFEST_NESTED_MANIFEST_FIELD: &str = "manifest";
const INVOCATION_SCHEMA: &str = "runx.external_adapter.invocation.v1";
const MANIFEST_SCHEMA: &str = "runx.external_adapter.manifest.v1";
const RESPONSE_SCHEMA: &str = "runx.external_adapter.response.v1";
const CREDENTIAL_REQUEST_SCHEMA: &str = "runx.external_adapter.credential_request.v1";
const CANCELLATION_SCHEMA: &str = "runx.external_adapter.cancellation.v1";
const POLL_INTERVAL: Duration = Duration::from_millis(10);
const FORCE_KILL_GRACE: Duration = Duration::from_millis(100);
const RESPONSE_LIMIT_BYTES: usize = 1024 * 1024;

#[derive(Clone, Debug, PartialEq)]
pub struct ExternalAdapterProcessOutcome {
    pub response: ExternalAdapterResponse,
    pub process_exit_code: Option<i32>,
    pub duration_ms: u64,
}

#[derive(Clone, Debug)]
pub struct ExternalAdapterSkillAdapter<
    R = InlineExternalAdapterManifestResolver,
    S = ExternalAdapterProcessSupervisor,
> {
    manifest_resolver: R,
    supervisor: S,
}

impl<R, S> ExternalAdapterSkillAdapter<R, S> {
    #[must_use]
    pub const fn new(manifest_resolver: R, supervisor: S) -> Self {
        Self {
            manifest_resolver,
            supervisor,
        }
    }
}

impl Default
    for ExternalAdapterSkillAdapter<
        InlineExternalAdapterManifestResolver,
        ExternalAdapterProcessSupervisor,
    >
{
    fn default() -> Self {
        Self::new(
            InlineExternalAdapterManifestResolver,
            ExternalAdapterProcessSupervisor,
        )
    }
}

impl<R, S> SkillAdapter for ExternalAdapterSkillAdapter<R, S>
where
    R: ExternalAdapterManifestResolver,
    S: ExternalAdapterSupervisor,
{
    fn adapter_type(&self) -> &'static str {
        "external-adapter"
    }

    fn invoke(&self, request: SkillInvocation) -> Result<SkillOutput, RuntimeError> {
        if request.source.source_type != "external-adapter" {
            return Err(RuntimeError::UnsupportedAdapter {
                adapter_type: request.source.source_type,
            });
        }
        let skill_name = request.skill_name.clone();
        invoke_external_adapter_skill(request, &self.manifest_resolver, &self.supervisor).map_err(
            |error| RuntimeError::SkillFailed {
                skill_name,
                message: error.to_string(),
            },
        )
    }
}

pub trait ExternalAdapterManifestResolver {
    fn resolve_manifest(
        &self,
        request: &SkillInvocation,
    ) -> Result<ExternalAdapterManifest, ExternalAdapterSkillAdapterError>;
}

pub trait ExternalAdapterSupervisor {
    fn invoke_external_adapter(
        &self,
        manifest: &ExternalAdapterManifest,
        invocation: &ExternalAdapterInvocation,
    ) -> Result<ExternalAdapterProcessOutcome, ExternalAdapterSupervisorError>;
}

impl ExternalAdapterSupervisor for ExternalAdapterProcessSupervisor {
    fn invoke_external_adapter(
        &self,
        manifest: &ExternalAdapterManifest,
        invocation: &ExternalAdapterInvocation,
    ) -> Result<ExternalAdapterProcessOutcome, ExternalAdapterSupervisorError> {
        ExternalAdapterProcessSupervisor::invoke(self, manifest, invocation)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct InlineExternalAdapterManifestResolver;

impl ExternalAdapterManifestResolver for InlineExternalAdapterManifestResolver {
    fn resolve_manifest(
        &self,
        request: &SkillInvocation,
    ) -> Result<ExternalAdapterManifest, ExternalAdapterSkillAdapterError> {
        let value = inline_manifest_value(&request.source.raw)
            .ok_or(ExternalAdapterSkillAdapterError::MissingInlineManifest)?;
        let JsonValue::Object(_) = value else {
            return Err(ExternalAdapterSkillAdapterError::InvalidInlineManifestShape);
        };
        manifest_from_value(value)
    }
}

#[derive(Debug, Error)]
pub enum ExternalAdapterSkillAdapterError {
    #[error(
        "external adapter source is missing an inline manifest at source.external_adapter.manifest or source.external_adapter_manifest"
    )]
    MissingInlineManifest,
    #[error("external adapter inline manifest must be an object")]
    InvalidInlineManifestShape,
    #[error("external adapter source metadata '{field}' must be a string when present")]
    InvalidSourceMetadata { field: &'static str },
    #[error(
        "external adapter response exit_code {actual} does not fit in a runtime process exit code"
    )]
    ExitCodeOutOfRange { actual: i64 },
    #[error("external adapter JSON failed while {context}: {source}")]
    Json {
        context: String,
        #[source]
        source: serde_json::Error,
    },
    #[error(transparent)]
    Supervisor(#[from] ExternalAdapterSupervisorError),
}

#[derive(Debug, Error)]
pub enum ExternalAdapterSupervisorError {
    #[error("external adapter manifest uses unsupported protocol version '{actual}'")]
    UnsupportedManifestProtocol { actual: String },
    #[error("external adapter invocation uses unsupported protocol version '{actual}'")]
    UnsupportedInvocationProtocol { actual: String },
    #[error("external adapter response uses unsupported protocol version '{actual}'")]
    UnsupportedResponseProtocol { actual: String },
    #[error("external adapter manifest schema '{actual}' is unsupported")]
    UnsupportedManifestSchema { actual: String },
    #[error("external adapter invocation schema '{actual}' is unsupported")]
    UnsupportedInvocationSchema { actual: String },
    #[error("external adapter response schema '{actual}' is unsupported")]
    UnsupportedResponseSchema { actual: String },
    #[error("external adapter manifest uses unsupported transport '{kind:?}'")]
    UnsupportedTransport { kind: ExternalAdapterTransportKind },
    #[error("external adapter process transport is missing command")]
    MissingProcessCommand,
    #[error("external adapter process command is empty")]
    EmptyProcessCommand,
    #[error(
        "external adapter invocation adapter id '{invocation_adapter_id}' does not match manifest adapter id '{manifest_adapter_id}'"
    )]
    AdapterIdMismatch {
        manifest_adapter_id: String,
        invocation_adapter_id: String,
    },
    #[error("external adapter '{adapter_id}' does not support source type '{source_type}'")]
    UnsupportedSourceType {
        adapter_id: String,
        source_type: String,
    },
    #[error("external adapter startup timeout must be greater than zero")]
    InvalidStartupTimeout,
    #[error("external adapter invocation timeout must be greater than zero")]
    InvalidInvocationTimeout,
    #[error("external adapter invocation env value '{key}' must be a string")]
    InvalidEnvValue { key: String },
    #[error("external adapter process timed out after {timeout_ms}ms")]
    TimedOut {
        timeout_ms: u64,
        cancellation: Box<ExternalAdapterCancellationFrame>,
    },
    #[error("external adapter process exited before returning an accepted response: {exit_status}")]
    ProcessFailed { exit_status: String },
    #[error("external adapter process returned no stdout response")]
    EmptyResponse,
    #[error("external adapter process response exceeded {limit_bytes} bytes")]
    ResponseTooLarge { limit_bytes: usize },
    #[error("external adapter process made an unexpected credential request '{request_id}'")]
    UnexpectedCredentialRequest { request_id: String },
    #[error("external adapter process returned unsupported frame schema '{schema}'")]
    UnsupportedFrameSchema { schema: String },
    #[error("external adapter response {field} was '{actual}', expected '{expected}'")]
    ResponseMismatch {
        field: &'static str,
        expected: String,
        actual: String,
    },
    #[error("external adapter process I/O failed while {context}: {source}")]
    Io {
        context: String,
        #[source]
        source: std::io::Error,
    },
    #[error("external adapter JSON failed while {context}: {source}")]
    Json {
        context: String,
        #[source]
        source: serde_json::Error,
    },
}

#[derive(Clone, Debug, Default)]
pub struct ExternalAdapterProcessSupervisor;

impl ExternalAdapterProcessSupervisor {
    pub fn invoke(
        &self,
        manifest: &ExternalAdapterManifest,
        invocation: &ExternalAdapterInvocation,
    ) -> Result<ExternalAdapterProcessOutcome, ExternalAdapterSupervisorError> {
        validate_invocation_contract(manifest, invocation)?;
        let started = Instant::now();
        let command = process_command(manifest)?;
        let mut child = spawn_process(command, manifest, invocation)?;
        let stdout = capture_pipe(child.stdout.take(), "opening external adapter stdout pipe")?;
        let stderr = capture_pipe(child.stderr.take(), "opening external adapter stderr pipe")?;
        if let Err(error) = write_invocation(&mut child, invocation) {
            let _cleanup = kill_timed_out_process(&mut child, KillSignal::Force);
            let _wait = child.wait();
            let _stdout = join_capture(stdout, "collecting failed external adapter stdout");
            let _stderr = join_capture(stderr, "collecting failed external adapter stderr");
            return Err(error);
        }
        let timeout = Duration::from_millis(manifest.timeouts.invocation_ms);
        let wait_result = wait_for_exit(&mut child, timeout)?;

        let status = match wait_result {
            WaitResult::Exited(status) => status,
            WaitResult::TimedOut => {
                let _stdout = join_capture(stdout, "collecting timed out external adapter stdout");
                let _stderr = join_capture(stderr, "collecting timed out external adapter stderr");
                return Err(ExternalAdapterSupervisorError::TimedOut {
                    timeout_ms: manifest.timeouts.invocation_ms,
                    cancellation: Box::new(timeout_cancellation_frame(
                        manifest,
                        invocation,
                        manifest.timeouts.invocation_ms,
                    )),
                });
            }
        };
        let stdout = join_capture(stdout, "collecting external adapter stdout")?;
        let _stderr = join_capture(stderr, "collecting external adapter stderr")?;
        if !status.success() {
            return Err(ExternalAdapterSupervisorError::ProcessFailed {
                exit_status: status.to_string(),
            });
        }
        if stdout.truncated {
            return Err(ExternalAdapterSupervisorError::ResponseTooLarge {
                limit_bytes: RESPONSE_LIMIT_BYTES,
            });
        }
        let response = parse_response(&stdout.bytes)?;
        validate_response_contract(invocation, &response)?;
        Ok(ExternalAdapterProcessOutcome {
            response,
            process_exit_code: status.code(),
            duration_ms: duration_ms(started),
        })
    }
}

fn invoke_external_adapter_skill<R, S>(
    request: SkillInvocation,
    manifest_resolver: &R,
    supervisor: &S,
) -> Result<SkillOutput, ExternalAdapterSkillAdapterError>
where
    R: ExternalAdapterManifestResolver,
    S: ExternalAdapterSupervisor,
{
    let manifest = manifest_resolver.resolve_manifest(&request)?;
    let invocation = skill_invocation_contract(&request, &manifest)?;
    let outcome = supervisor.invoke_external_adapter(&manifest, &invocation)?;
    skill_output_from_outcome(outcome)
}

fn inline_manifest_value(source: &JsonObject) -> Option<&JsonValue> {
    source.get(MANIFEST_INLINE_FIELD).or_else(|| {
        let JsonValue::Object(external_adapter) = source.get(MANIFEST_NESTED_FIELD)? else {
            return None;
        };
        external_adapter.get(MANIFEST_NESTED_MANIFEST_FIELD)
    })
}

fn manifest_from_value(
    value: &JsonValue,
) -> Result<ExternalAdapterManifest, ExternalAdapterSkillAdapterError> {
    let value = serde_json::to_value(value).map_err(|source| {
        json_adapter_error("serializing external adapter inline manifest", source)
    })?;
    serde_json::from_value(value)
        .map_err(|source| json_adapter_error("validating external adapter inline manifest", source))
}

fn skill_invocation_contract(
    request: &SkillInvocation,
    manifest: &ExternalAdapterManifest,
) -> Result<ExternalAdapterInvocation, ExternalAdapterSkillAdapterError> {
    let invocation_id = optional_source_string(&request.source.raw, "invocation_id")?
        .unwrap_or_else(|| {
            format!(
                "external_adapter.{}.invoke",
                identifier_segment(&request.skill_name)
            )
        });
    let run_id = optional_source_string(&request.source.raw, "run_id")?
        .unwrap_or_else(|| format!("run_{}", identifier_segment(&request.skill_name)));
    let step_id = optional_source_string(&request.source.raw, "step_id")?
        .unwrap_or_else(|| identifier_segment(&request.skill_name));
    let skill_ref = optional_source_string(&request.source.raw, "skill_ref")?
        .unwrap_or_else(|| request.skill_name.clone());
    Ok(ExternalAdapterInvocation {
        schema: INVOCATION_SCHEMA.to_owned(),
        protocol_version: EXTERNAL_ADAPTER_PROTOCOL_VERSION.to_owned(),
        invocation_id,
        adapter_id: manifest.adapter_id.clone(),
        run_id: run_id.clone(),
        step_id,
        source_type: request.source.source_type.clone(),
        skill_ref,
        harness_ref: reference(ReferenceType::Harness, &format!("runx:harness:{run_id}")),
        host_ref: reference(ReferenceType::Host, "runx:host:runtime"),
        inputs: request.inputs.clone(),
        resolved_inputs: (!request.resolved_inputs.is_empty())
            .then(|| request.resolved_inputs.clone()),
        cwd: Some(invocation_cwd(request)),
        receipt_dir: request.env.get(RUNX_RECEIPT_DIR_ENV).cloned(),
        env: invocation_env(&request.env),
        credential_refs: None,
        metadata: None,
    })
}

fn optional_source_string(
    source: &JsonObject,
    field: &'static str,
) -> Result<Option<String>, ExternalAdapterSkillAdapterError> {
    match source.get(field) {
        Some(JsonValue::String(value)) => Ok(Some(value.clone())),
        Some(_) => Err(ExternalAdapterSkillAdapterError::InvalidSourceMetadata { field }),
        None => Ok(None),
    }
}

fn invocation_cwd(request: &SkillInvocation) -> String {
    let Some(cwd) = request.source.cwd.as_ref() else {
        return request.skill_directory.to_string_lossy().into_owned();
    };
    let path = Path::new(cwd);
    if path.is_absolute() {
        return cwd.clone();
    }
    request
        .skill_directory
        .join(PathBuf::from(cwd))
        .to_string_lossy()
        .into_owned()
}

fn invocation_env(env: &BTreeMap<String, String>) -> Option<JsonObject> {
    (!env.is_empty()).then(|| {
        env.iter()
            .map(|(key, value)| (key.clone(), JsonValue::String(value.clone())))
            .collect()
    })
}

fn skill_output_from_outcome(
    outcome: ExternalAdapterProcessOutcome,
) -> Result<SkillOutput, ExternalAdapterSkillAdapterError> {
    let response = outcome.response;
    let status = runtime_status(&response.status);
    let stdout = response_stdout(&response)?;
    let stderr = response.stderr.clone().unwrap_or_default();
    let exit_code = response_exit_code(&response)?;
    let mut metadata = response.metadata.clone().unwrap_or_default();
    metadata.insert(
        "adapter_id".to_owned(),
        JsonValue::String(response.adapter_id.clone()),
    );
    metadata.insert(
        "external_adapter_status".to_owned(),
        JsonValue::String(external_adapter_status_label(&response.status).to_owned()),
    );
    if let Some(process_exit_code) = outcome.process_exit_code {
        metadata.insert(
            "process_exit_code".to_owned(),
            JsonValue::Number(JsonNumber::I64(i64::from(process_exit_code))),
        );
    }

    Ok(SkillOutput {
        status,
        stdout,
        stderr,
        exit_code,
        duration_ms: outcome.duration_ms,
        metadata,
    })
}

fn runtime_status(status: &ExternalAdapterStatus) -> InvocationStatus {
    match status {
        ExternalAdapterStatus::Completed => InvocationStatus::Success,
        ExternalAdapterStatus::Failed
        | ExternalAdapterStatus::HostResolutionRequested
        | ExternalAdapterStatus::Cancelled => InvocationStatus::Failure,
    }
}

fn response_stdout(
    response: &ExternalAdapterResponse,
) -> Result<String, ExternalAdapterSkillAdapterError> {
    if let Some(stdout) = response.stdout.clone() {
        return Ok(stdout);
    }
    let Some(output) = response.output.as_ref() else {
        return Ok(String::new());
    };
    serde_json::to_string(&JsonValue::Object(output.clone()))
        .map_err(|source| json_adapter_error("serializing external adapter output", source))
}

fn response_exit_code(
    response: &ExternalAdapterResponse,
) -> Result<Option<i32>, ExternalAdapterSkillAdapterError> {
    let Some(exit_code) = response.exit_code.flatten() else {
        return Ok(None);
    };
    i32::try_from(exit_code)
        .map(Some)
        .map_err(|_| ExternalAdapterSkillAdapterError::ExitCodeOutOfRange { actual: exit_code })
}

fn external_adapter_status_label(status: &ExternalAdapterStatus) -> &'static str {
    match status {
        ExternalAdapterStatus::Completed => "completed",
        ExternalAdapterStatus::Failed => "failed",
        ExternalAdapterStatus::HostResolutionRequested => "host_resolution_requested",
        ExternalAdapterStatus::Cancelled => "cancelled",
    }
}

fn reference(reference_type: ReferenceType, uri: &str) -> Reference {
    Reference {
        reference_type,
        uri: uri.to_owned(),
        provider: None,
        locator: None,
        label: None,
        observed_at: None,
        proof_kind: None,
    }
}

fn identifier_segment(value: &str) -> String {
    normalize_request_id(value)
        .trim_matches(['.', '_', '-'])
        .replace('.', "-")
}

fn normalize_request_id(value: &str) -> String {
    let mut normalized = String::new();
    let mut replaced = false;
    for character in value.chars() {
        if character.is_ascii_alphanumeric() || matches!(character, '_' | '.' | '-') {
            normalized.push(character);
            replaced = false;
        } else if !replaced {
            normalized.push('_');
            replaced = true;
        }
    }
    if normalized.trim_matches(['.', '_', '-']).is_empty() {
        return "skill".to_owned();
    }
    normalized
}

fn validate_invocation_contract(
    manifest: &ExternalAdapterManifest,
    invocation: &ExternalAdapterInvocation,
) -> Result<(), ExternalAdapterSupervisorError> {
    if manifest.schema != MANIFEST_SCHEMA {
        return Err(ExternalAdapterSupervisorError::UnsupportedManifestSchema {
            actual: manifest.schema.clone(),
        });
    }
    if invocation.schema != INVOCATION_SCHEMA {
        return Err(
            ExternalAdapterSupervisorError::UnsupportedInvocationSchema {
                actual: invocation.schema.clone(),
            },
        );
    }
    if manifest.protocol_version != EXTERNAL_ADAPTER_PROTOCOL_VERSION {
        return Err(
            ExternalAdapterSupervisorError::UnsupportedManifestProtocol {
                actual: manifest.protocol_version.clone(),
            },
        );
    }
    if invocation.protocol_version != EXTERNAL_ADAPTER_PROTOCOL_VERSION {
        return Err(
            ExternalAdapterSupervisorError::UnsupportedInvocationProtocol {
                actual: invocation.protocol_version.clone(),
            },
        );
    }
    if manifest.adapter_id != invocation.adapter_id {
        return Err(ExternalAdapterSupervisorError::AdapterIdMismatch {
            manifest_adapter_id: manifest.adapter_id.clone(),
            invocation_adapter_id: invocation.adapter_id.clone(),
        });
    }
    if !manifest
        .supported_source_types
        .iter()
        .any(|source_type| source_type == &invocation.source_type)
    {
        return Err(ExternalAdapterSupervisorError::UnsupportedSourceType {
            adapter_id: manifest.adapter_id.clone(),
            source_type: invocation.source_type.clone(),
        });
    }
    if manifest.timeouts.startup_ms == 0 {
        return Err(ExternalAdapterSupervisorError::InvalidStartupTimeout);
    }
    if manifest.timeouts.invocation_ms == 0 {
        return Err(ExternalAdapterSupervisorError::InvalidInvocationTimeout);
    }
    if manifest.transport.kind != ExternalAdapterTransportKind::Process {
        return Err(ExternalAdapterSupervisorError::UnsupportedTransport {
            kind: manifest.transport.kind.clone(),
        });
    }
    Ok(())
}

fn process_command(
    manifest: &ExternalAdapterManifest,
) -> Result<&str, ExternalAdapterSupervisorError> {
    let command = manifest
        .transport
        .command
        .as_deref()
        .ok_or(ExternalAdapterSupervisorError::MissingProcessCommand)?;
    if command.trim().is_empty() {
        return Err(ExternalAdapterSupervisorError::EmptyProcessCommand);
    }
    Ok(command)
}

fn spawn_process(
    process_command: &str,
    manifest: &ExternalAdapterManifest,
    invocation: &ExternalAdapterInvocation,
) -> Result<Child, ExternalAdapterSupervisorError> {
    let mut command = Command::new(process_command);
    if let Some(args) = manifest.transport.args.as_ref() {
        command.args(args);
    }
    if let Some(cwd) = invocation.cwd.as_ref() {
        command.current_dir(cwd);
    }
    command
        .env_clear()
        .envs(process_env(invocation)?)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    configure_process_group(&mut command);
    command
        .spawn()
        .map_err(|source| io_error("spawning external adapter process", source))
}

fn process_env(
    invocation: &ExternalAdapterInvocation,
) -> Result<BTreeMap<String, String>, ExternalAdapterSupervisorError> {
    let mut env = BTreeMap::new();
    if let Some(scoped_env) = invocation.env.as_ref() {
        for (key, value) in scoped_env {
            let JsonValue::String(value) = value else {
                return Err(ExternalAdapterSupervisorError::InvalidEnvValue { key: key.clone() });
            };
            env.insert(key.clone(), value.clone());
        }
    }
    if let Some(receipt_dir) = invocation.receipt_dir.as_ref() {
        env.insert("RUNX_RECEIPT_DIR".to_owned(), receipt_dir.clone());
    }
    Ok(env)
}

fn write_invocation(
    child: &mut Child,
    invocation: &ExternalAdapterInvocation,
) -> Result<(), ExternalAdapterSupervisorError> {
    let Some(mut stdin) = child.stdin.take() else {
        return Ok(());
    };
    serde_json::to_writer(&mut stdin, invocation)
        .map_err(|source| json_error("serializing external adapter invocation", source))?;
    stdin
        .write_all(b"\n")
        .map_err(|source| io_error("writing external adapter invocation", source))?;
    Ok(())
}

fn parse_response(bytes: &[u8]) -> Result<ExternalAdapterResponse, ExternalAdapterSupervisorError> {
    let bytes = trim_ascii_whitespace(bytes);
    if bytes.is_empty() {
        return Err(ExternalAdapterSupervisorError::EmptyResponse);
    }
    let frame: ExternalAdapterFrameSchema = serde_json::from_slice(bytes)
        .map_err(|source| json_error("parsing external adapter response frame", source))?;
    match frame.schema.as_str() {
        RESPONSE_SCHEMA => serde_json::from_slice(bytes)
            .map_err(|source| json_error("validating external adapter response frame", source)),
        CREDENTIAL_REQUEST_SCHEMA => {
            let request: ExternalAdapterCredentialRequest =
                serde_json::from_slice(bytes).map_err(|source| {
                    json_error(
                        "validating unexpected external adapter credential request",
                        source,
                    )
                })?;
            Err(
                ExternalAdapterSupervisorError::UnexpectedCredentialRequest {
                    request_id: request.request_id,
                },
            )
        }
        other => Err(ExternalAdapterSupervisorError::UnsupportedFrameSchema {
            schema: other.to_owned(),
        }),
    }
}

#[derive(Debug, serde::Deserialize)]
struct ExternalAdapterFrameSchema {
    schema: String,
}

fn validate_response_contract(
    invocation: &ExternalAdapterInvocation,
    response: &ExternalAdapterResponse,
) -> Result<(), ExternalAdapterSupervisorError> {
    if response.schema != RESPONSE_SCHEMA {
        return Err(ExternalAdapterSupervisorError::UnsupportedResponseSchema {
            actual: response.schema.clone(),
        });
    }
    if response.protocol_version != EXTERNAL_ADAPTER_PROTOCOL_VERSION {
        return Err(
            ExternalAdapterSupervisorError::UnsupportedResponseProtocol {
                actual: response.protocol_version.clone(),
            },
        );
    }
    if response.adapter_id != invocation.adapter_id {
        return Err(ExternalAdapterSupervisorError::ResponseMismatch {
            field: "adapter_id",
            expected: invocation.adapter_id.clone(),
            actual: response.adapter_id.clone(),
        });
    }
    if response.invocation_id != invocation.invocation_id {
        return Err(ExternalAdapterSupervisorError::ResponseMismatch {
            field: "invocation_id",
            expected: invocation.invocation_id.clone(),
            actual: response.invocation_id.clone(),
        });
    }
    Ok(())
}

fn timeout_cancellation_frame(
    manifest: &ExternalAdapterManifest,
    invocation: &ExternalAdapterInvocation,
    timeout_ms: u64,
) -> ExternalAdapterCancellationFrame {
    ExternalAdapterCancellationFrame {
        schema: CANCELLATION_SCHEMA.to_owned(),
        protocol_version: EXTERNAL_ADAPTER_PROTOCOL_VERSION.to_owned(),
        frame_id: format!("{}_timeout_cancel", invocation.invocation_id),
        invocation_id: invocation.invocation_id.clone(),
        adapter_id: manifest.adapter_id.clone(),
        reason: format!("invocation timeout after {timeout_ms}ms"),
        requested_at: now_iso8601(),
    }
}

fn capture_pipe<R>(
    pipe: Option<R>,
    context: &'static str,
) -> Result<JoinHandle<std::io::Result<CapturedOutput>>, ExternalAdapterSupervisorError>
where
    R: Read + Send + 'static,
{
    pipe.map(capture_stream)
        .ok_or_else(|| io_error(context, std::io::Error::other("pipe was not captured")))
}

fn capture_stream<R>(mut reader: R) -> JoinHandle<std::io::Result<CapturedOutput>>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut captured = Vec::new();
        let mut truncated = false;
        let mut buffer = [0_u8; 8192];
        loop {
            let count = reader.read(&mut buffer)?;
            if count == 0 {
                return Ok(CapturedOutput {
                    bytes: captured,
                    truncated,
                });
            }
            let remaining = RESPONSE_LIMIT_BYTES.saturating_sub(captured.len());
            if remaining > 0 {
                captured.extend_from_slice(&buffer[..count.min(remaining)]);
            }
            if count > remaining {
                truncated = true;
            }
        }
    })
}

fn join_capture(
    handle: JoinHandle<std::io::Result<CapturedOutput>>,
    context: &'static str,
) -> Result<CapturedOutput, ExternalAdapterSupervisorError> {
    match handle.join() {
        Ok(Ok(output)) => Ok(output),
        Ok(Err(source)) => Err(io_error(context, source)),
        Err(_) => Err(io_error(
            context,
            std::io::Error::other("output reader thread failed"),
        )),
    }
}

fn wait_for_exit(
    child: &mut Child,
    timeout: Duration,
) -> Result<WaitResult, ExternalAdapterSupervisorError> {
    let started = Instant::now();
    loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|source| io_error("polling external adapter process", source))?
        {
            return Ok(WaitResult::Exited(status));
        }
        if started.elapsed() >= timeout {
            kill_timed_out_process(child, KillSignal::Terminate)?;
            thread::sleep(FORCE_KILL_GRACE);
            kill_timed_out_process(child, KillSignal::Force)?;
            child.wait().map_err(|source| {
                io_error("waiting for timed out external adapter process", source)
            })?;
            return Ok(WaitResult::TimedOut);
        }
        thread::sleep(POLL_INTERVAL);
    }
}

#[cfg(unix)]
fn configure_process_group(command: &mut Command) {
    command.process_group(0);
}

#[cfg(not(unix))]
fn configure_process_group(_command: &mut Command) {}

enum KillSignal {
    Terminate,
    Force,
}

impl KillSignal {
    #[cfg(unix)]
    fn kill_arg(&self) -> &'static str {
        match self {
            Self::Terminate => "-TERM",
            Self::Force => "-KILL",
        }
    }
}

#[cfg(unix)]
fn kill_timed_out_process(
    child: &mut Child,
    signal: KillSignal,
) -> Result<(), ExternalAdapterSupervisorError> {
    let process_group = format!("-{}", child.id());
    let status = Command::new("/bin/kill")
        .arg(signal.kill_arg())
        .arg(&process_group)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    if status.is_ok_and(|status| status.success()) {
        return Ok(());
    }
    if child
        .try_wait()
        .map_err(|source| io_error("polling timed out external adapter process", source))?
        .is_some()
    {
        return Ok(());
    }
    kill_direct_child_if_running(child)
}

#[cfg(not(unix))]
fn kill_timed_out_process(
    child: &mut Child,
    _signal: KillSignal,
) -> Result<(), ExternalAdapterSupervisorError> {
    kill_direct_child_if_running(child)
}

fn kill_direct_child_if_running(child: &mut Child) -> Result<(), ExternalAdapterSupervisorError> {
    if child
        .try_wait()
        .map_err(|source| io_error("polling timed out external adapter process", source))?
        .is_some()
    {
        return Ok(());
    }
    child
        .kill()
        .map_err(|source| io_error("killing timed out external adapter process", source))
}

fn trim_ascii_whitespace(bytes: &[u8]) -> &[u8] {
    let start = bytes
        .iter()
        .position(|byte| !byte.is_ascii_whitespace())
        .unwrap_or(bytes.len());
    let end = bytes
        .iter()
        .rposition(|byte| !byte.is_ascii_whitespace())
        .map_or(start, |index| index + 1);
    &bytes[start..end]
}

fn duration_ms(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX)
}

fn io_error(context: impl Into<String>, source: std::io::Error) -> ExternalAdapterSupervisorError {
    ExternalAdapterSupervisorError::Io {
        context: context.into(),
        source,
    }
}

fn json_error(
    context: impl Into<String>,
    source: serde_json::Error,
) -> ExternalAdapterSupervisorError {
    ExternalAdapterSupervisorError::Json {
        context: context.into(),
        source,
    }
}

fn json_adapter_error(
    context: impl Into<String>,
    source: serde_json::Error,
) -> ExternalAdapterSkillAdapterError {
    ExternalAdapterSkillAdapterError::Json {
        context: context.into(),
        source,
    }
}

struct CapturedOutput {
    bytes: Vec<u8>,
    truncated: bool,
}

enum WaitResult {
    Exited(ExitStatus),
    TimedOut,
}
