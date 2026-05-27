use runx_contracts::{
    ExecutionSemantics, GovernedDisposition, InputContextCapture, JsonObject, JsonValue,
    OutcomeState, ReceiptOutcome, ReceiptSurfaceRef,
};
use runx_core::policy::{
    CwdPolicy, SandboxDeclaration, SandboxProfile, normalize_sandbox_declaration,
};
use std::collections::BTreeMap;

use crate::ValidationError;
use crate::graph::{RawGraphIr, validate_graph_document};

mod catalog;
mod fixtures;
mod markdown;
mod runner_definition;
mod types;

pub use markdown::{extract_skill_quality_profile, parse_skill_markdown};
pub use types::{
    CatalogAudience, CatalogKind, CatalogMetadata, CatalogVisibility, HarnessCallerFixture,
    HarnessExpectation, InputMode, RawSkillIr, ReceiptExpectation, RunnerHarnessCase,
    RunnerHarnessManifest, SkillArtifactContract, SkillIdempotencyPolicy, SkillInput,
    SkillMcpServer, SkillQualityProfile, SkillRetryPolicy, SkillRunnerDefinition, SkillSandbox,
    SkillSource, SourceKind, ValidateSkillMode, ValidateSkillOptions, ValidatedSkill,
};

pub(crate) use catalog::validate_catalog_metadata;
pub(crate) use fixtures::validate_harness_manifest;
pub(crate) use runner_definition::validate_runner_definition;

struct SkillGovernance {
    retry: Option<SkillRetryPolicy>,
    idempotency: Option<SkillIdempotencyPolicy>,
    mutating: Option<bool>,
    artifacts: Option<SkillArtifactContract>,
    allowed_tools: Option<Vec<String>>,
    execution: Option<ExecutionSemantics>,
}

pub fn validate_skill(raw: RawSkillIr) -> Result<ValidatedSkill, ValidationError> {
    validate_skill_with_options(raw, ValidateSkillOptions::default())
}

pub fn validate_skill_with_options(
    raw: RawSkillIr,
    options: ValidateSkillOptions,
) -> Result<ValidatedSkill, ValidationError> {
    let runx = validate_runx_metadata(raw.frontmatter.get("runx"), options.mode)?;
    let source = raw
        .frontmatter
        .get("source")
        .map(|value| optional_object(Some(value), "source"))
        .transpose()?
        .flatten()
        .unwrap_or_else(default_agent_source);
    let risk = raw.frontmatter.get("risk").cloned();
    let governance = validate_skill_governance(&raw, runx.as_ref(), risk.as_ref())?;

    Ok(ValidatedSkill {
        name: required_string(raw.frontmatter.get("name"), "name")?,
        description: optional_string(raw.frontmatter.get("description"), "description")?,
        body: raw.body.clone(),
        source: validate_source(&source, runx.as_ref())?,
        inputs: validate_inputs(
            optional_object(raw.frontmatter.get("inputs"), "inputs")?.unwrap_or_default(),
        )?,
        auth: raw.frontmatter.get("auth").cloned(),
        risk: risk.clone(),
        runtime: raw.frontmatter.get("runtime").cloned(),
        retry: governance.retry,
        idempotency: governance.idempotency,
        mutating: governance.mutating,
        artifacts: governance.artifacts,
        quality_profile: extract_skill_quality_profile(&raw.body),
        allowed_tools: governance.allowed_tools,
        execution: governance.execution,
        runx,
        raw,
    })
}

fn validate_runx_metadata(
    value: Option<&JsonValue>,
    mode: ValidateSkillMode,
) -> Result<Option<JsonObject>, ValidationError> {
    match value {
        None | Some(JsonValue::Null) => Ok(None),
        Some(JsonValue::Object(value)) => Ok(Some(value.clone())),
        Some(_) if mode == ValidateSkillMode::Lenient => Ok(None),
        Some(_) => Err(ValidationError::InvalidField {
            field: "runx".to_owned(),
            message: "runx must be an object when present.".to_owned(),
        }),
    }
}

fn validate_skill_governance(
    raw: &RawSkillIr,
    runx: Option<&JsonObject>,
    risk: Option<&JsonValue>,
) -> Result<SkillGovernance, ValidationError> {
    Ok(SkillGovernance {
        retry: validate_retry(
            first_value(raw.frontmatter.get("retry"), field_value(runx, "retry")),
            "retry",
        )?,
        idempotency: validate_idempotency(
            first_value(
                raw.frontmatter.get("idempotency"),
                field_value(runx, "idempotency"),
            ),
            "idempotency",
        )?,
        mutating: validate_mutating(
            first_value(
                first_value(
                    raw.frontmatter.get("mutating"),
                    nested_value(risk, "mutating"),
                ),
                field_value(runx, "mutating"),
            ),
            "mutating",
        )?,
        artifacts: validate_artifact_contract(field_value(runx, "artifacts"), "runx.artifacts")?,
        allowed_tools: validate_allowed_tools(
            field_value(runx, "allowed_tools"),
            "runx.allowed_tools",
        )?,
        execution: validate_execution_semantics(
            first_value(
                raw.frontmatter.get("execution"),
                field_value(runx, "execution"),
            ),
            "execution",
        )?,
    })
}

pub fn validate_skill_source(
    source: &JsonObject,
    runx: Option<&JsonObject>,
) -> Result<SkillSource, ValidationError> {
    validate_source(source, runx)
}

pub fn validate_skill_artifact_contract(
    value: Option<&JsonValue>,
    field: &str,
) -> Result<Option<SkillArtifactContract>, ValidationError> {
    validate_artifact_contract(value, field)
}

fn validate_source(
    source: &JsonObject,
    runx: Option<&JsonObject>,
) -> Result<SkillSource, ValidationError> {
    let source_type = required_string(source.get("type"), "source.type")?;
    let args = optional_string_array(source.get("args"), "source.args")?.unwrap_or_default();
    let input_mode = optional_input_mode(source.get("input_mode"))?;
    let timeout_seconds = optional_u64(source.get("timeout_seconds"), "source.timeout_seconds")?;

    if source_type == "cli-tool" {
        required_string(source.get("command"), "source.command")?;
    }
    validate_agent_command_boundary(source, &source_type)?;
    let source_kind = parse_source_kind(&source_type, "source.type")?;

    Ok(SkillSource {
        command: optional_string(source.get("command"), "source.command")?,
        args,
        cwd: optional_string(source.get("cwd"), "source.cwd")?,
        timeout_seconds,
        input_mode,
        sandbox: validate_sandbox(first_value(
            source.get("sandbox"),
            field_value(runx, "sandbox"),
        ))?,
        server: validate_mcp_server(source, &source_type)?,
        catalog_ref: validate_catalog_ref(source, &source_type)?,
        tool: validate_mcp_tool(source, &source_type)?,
        arguments: optional_object(source.get("arguments"), "source.arguments")?,
        agent_card_url: validate_a2a_url(source, &source_type)?,
        agent_identity: optional_string(source.get("agent_identity"), "source.agent_identity")?,
        agent: validate_agent(source, &source_type)?,
        task: validate_task(source, &source_type)?,
        hook: validate_hook(source, &source_type)?,
        outputs: optional_object(source.get("outputs"), "source.outputs")?,
        graph: validate_graph_source(source, &source_type)?,
        raw: source.clone(),
        source_type: source_kind,
    })
}

fn parse_source_kind(value: &str, field: &str) -> Result<SourceKind, ValidationError> {
    match value {
        "cli-tool" => Ok(SourceKind::CliTool),
        "mcp" => Ok(SourceKind::Mcp),
        "catalog" => Ok(SourceKind::Catalog),
        "a2a" => Ok(SourceKind::A2a),
        "agent" => Ok(SourceKind::Agent),
        "agent-step" => Ok(SourceKind::AgentStep),
        "harness-hook" => Ok(SourceKind::HarnessHook),
        "graph" => Ok(SourceKind::Graph),
        "external-adapter" => Ok(SourceKind::ExternalAdapter),
        other => Err(validation_error(format!(
            "{field} {other} is not a supported source type."
        ))),
    }
}

fn validate_sandbox(value: Option<&JsonValue>) -> Result<Option<SkillSandbox>, ValidationError> {
    let Some(record) = value else {
        return Ok(None);
    };
    let record = required_object(Some(record), "sandbox")?;
    let profile = required_sandbox_profile(record.get("profile"), "sandbox.profile")?;
    let cwd_policy = optional_cwd_policy(record.get("cwd_policy"))?;
    let env_allowlist =
        optional_string_array(record.get("env_allowlist"), "sandbox.env_allowlist")?;
    let network = optional_bool(record.get("network"), "sandbox.network")?;
    let writable_paths =
        optional_string_array(record.get("writable_paths"), "sandbox.writable_paths")?
            .unwrap_or_default();
    let require_enforcement = optional_bool(
        record.get("require_enforcement"),
        "sandbox.require_enforcement",
    )?;
    let declaration = sandbox_declaration(
        &profile,
        cwd_policy.as_deref(),
        env_allowlist.clone(),
        network,
        Some(writable_paths.clone()),
        require_enforcement,
    )?;
    let normalized = normalize_sandbox_declaration(Some(&declaration));
    Ok(Some(SkillSandbox {
        profile: normalized.profile,
        cwd_policy: Some(normalized.cwd_policy),
        env_allowlist: normalized.env_allowlist,
        network: Some(normalized.network),
        writable_paths: normalized.writable_paths,
        require_enforcement: Some(normalized.require_enforcement),
        // TS currently preserves approvedEscalation only inside raw.
        approved_escalation: None,
        raw: record.clone(),
    }))
}

fn validate_execution_semantics(
    value: Option<&JsonValue>,
    field: &str,
) -> Result<Option<ExecutionSemantics>, ValidationError> {
    let Some(record) = optional_object(value, field)? else {
        return Ok(None);
    };
    Ok(Some(ExecutionSemantics {
        disposition: optional_disposition(
            record.get("disposition"),
            &format!("{field}.disposition"),
        )?,
        outcome_state: optional_outcome_state(
            record.get("outcome_state"),
            &format!("{field}.outcome_state"),
        )?,
        outcome: validate_outcome(record.get("outcome"), &format!("{field}.outcome"))?,
        input_context: validate_input_context(
            record.get("input_context"),
            &format!("{field}.input_context"),
        )?,
        surface_refs: validate_surface_refs(
            record.get("surface_refs"),
            &format!("{field}.surface_refs"),
        )?,
        evidence_refs: validate_surface_refs(
            record.get("evidence_refs"),
            &format!("{field}.evidence_refs"),
        )?,
    }))
}

fn validate_artifact_contract(
    value: Option<&JsonValue>,
    field: &str,
) -> Result<Option<SkillArtifactContract>, ValidationError> {
    let Some(record) = optional_object(value, field)? else {
        return Ok(None);
    };
    let emits = match record.get("emits") {
        Some(JsonValue::String(value)) => Some(vec![value.clone()]),
        value => optional_string_array(value, &format!("{field}.emits"))?,
    };
    let named_emits = validate_named_emits(
        first_value(record.get("named_emits"), record.get("namedEmits")),
        &format!("{field}.named_emits"),
    )?;
    let wrap_as = optional_non_empty_string(
        first_value(record.get("wrap_as"), record.get("wrapAs")),
        &format!("{field}.wrap_as"),
    )?;
    if emits.is_none() && named_emits.is_none() && wrap_as.is_none() {
        return Ok(None);
    }
    Ok(Some(SkillArtifactContract {
        emits,
        named_emits,
        wrap_as,
    }))
}

fn validate_inputs(inputs: JsonObject) -> Result<BTreeMap<String, SkillInput>, ValidationError> {
    inputs
        .into_iter()
        .map(|(name, value)| {
            let field = format!("inputs.{name}");
            let input = required_object(Some(&value), &field)?;
            Ok((
                name.clone(),
                SkillInput {
                    input_type: optional_string(input.get("type"), &format!("{field}.type"))?
                        .unwrap_or_else(|| "string".to_owned()),
                    required: optional_bool(input.get("required"), &format!("{field}.required"))?
                        .unwrap_or(false),
                    description: optional_string(
                        input.get("description"),
                        &format!("{field}.description"),
                    )?,
                    default: input.get("default").cloned(),
                },
            ))
        })
        .collect()
}

fn validate_retry(
    value: Option<&JsonValue>,
    field: &str,
) -> Result<Option<SkillRetryPolicy>, ValidationError> {
    let Some(retry) = optional_object(value, field)? else {
        return Ok(None);
    };
    let max_attempts =
        optional_u64(retry.get("max_attempts"), &format!("{field}.max_attempts"))?.unwrap_or(1);
    if max_attempts == 0 {
        return Err(validation_error(format!(
            "{field}.max_attempts must be a positive integer."
        )));
    }
    Ok(Some(SkillRetryPolicy { max_attempts }))
}

fn validate_idempotency(
    value: Option<&JsonValue>,
    field: &str,
) -> Result<Option<SkillIdempotencyPolicy>, ValidationError> {
    match value {
        None | Some(JsonValue::Null) => Ok(None),
        Some(JsonValue::String(value)) if value.trim().is_empty() => {
            Err(validation_error(format!("{field} must not be empty.")))
        }
        Some(JsonValue::String(value)) => Ok(Some(SkillIdempotencyPolicy {
            key: Some(value.clone()),
        })),
        Some(value) => {
            let record = required_object(Some(value), field)?;
            Ok(Some(SkillIdempotencyPolicy {
                key: optional_non_empty_string(record.get("key"), &format!("{field}.key"))?,
            }))
        }
    }
}

fn validate_mutating(
    value: Option<&JsonValue>,
    field: &str,
) -> Result<Option<bool>, ValidationError> {
    optional_bool(value, field)
}

fn validation_error(message: impl Into<String>) -> ValidationError {
    ValidationError::InvalidField {
        field: "skill".to_owned(),
        message: message.into(),
    }
}

fn required_string(value: Option<&JsonValue>, field: &str) -> Result<String, ValidationError> {
    match optional_string(value, field)? {
        Some(value) if !value.is_empty() => Ok(value),
        _ => Err(ValidationError::MissingField {
            field: field.to_owned(),
        }),
    }
}

fn optional_string(
    value: Option<&JsonValue>,
    field: &str,
) -> Result<Option<String>, ValidationError> {
    match value {
        None | Some(JsonValue::Null) => Ok(None),
        Some(JsonValue::String(value)) => Ok(Some(value.clone())),
        Some(_) => Err(validation_error(format!("{field} must be a string."))),
    }
}

fn optional_non_empty_string(
    value: Option<&JsonValue>,
    field: &str,
) -> Result<Option<String>, ValidationError> {
    let Some(value) = optional_string(value, field)? else {
        return Ok(None);
    };
    if value.trim().is_empty() {
        return Err(validation_error(format!("{field} must not be empty.")));
    }
    Ok(Some(value))
}

fn required_object<'a>(
    value: Option<&'a JsonValue>,
    field: &str,
) -> Result<&'a JsonObject, ValidationError> {
    match value {
        Some(JsonValue::Object(value)) => Ok(value),
        None | Some(JsonValue::Null) => Err(validation_error(format!("{field} is required."))),
        Some(_) => Err(validation_error(format!("{field} must be an object."))),
    }
}

fn optional_object(
    value: Option<&JsonValue>,
    field: &str,
) -> Result<Option<JsonObject>, ValidationError> {
    match value {
        None | Some(JsonValue::Null) => Ok(None),
        Some(JsonValue::Object(value)) => Ok(Some(value.clone())),
        Some(_) => Err(validation_error(format!("{field} must be an object."))),
    }
}

fn required_plain_array<'a>(
    value: Option<&'a JsonValue>,
    field: &str,
) -> Result<&'a [JsonValue], ValidationError> {
    match value {
        Some(JsonValue::Array(values)) => Ok(values),
        None | Some(JsonValue::Null) => Err(validation_error(format!("{field} is required."))),
        Some(_) => Err(validation_error(format!("{field} must be an array."))),
    }
}

fn optional_string_array(
    value: Option<&JsonValue>,
    field: &str,
) -> Result<Option<Vec<String>>, ValidationError> {
    match value {
        None | Some(JsonValue::Null) => Ok(None),
        Some(JsonValue::Array(values)) => values
            .iter()
            .map(|value| match value {
                JsonValue::String(value) => Ok(value.clone()),
                _ => Err(validation_error(format!(
                    "{field} must be an array of strings."
                ))),
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Some),
        Some(_) => Err(validation_error(format!(
            "{field} must be an array of strings."
        ))),
    }
}

fn optional_bool(value: Option<&JsonValue>, field: &str) -> Result<Option<bool>, ValidationError> {
    match value {
        None | Some(JsonValue::Null) => Ok(None),
        Some(JsonValue::Bool(value)) => Ok(Some(*value)),
        Some(_) => Err(validation_error(format!("{field} must be a boolean."))),
    }
}

fn optional_u64(value: Option<&JsonValue>, field: &str) -> Result<Option<u64>, ValidationError> {
    match value {
        None | Some(JsonValue::Null) => Ok(None),
        Some(JsonValue::Number(number)) => {
            let Some(value) = number.as_f64() else {
                return Err(validation_error(format!(
                    "{field} must be a finite number."
                )));
            };
            if value.fract() == 0.0 && value >= 0.0 && value <= u64::MAX as f64 {
                Ok(Some(value as u64))
            } else {
                Err(validation_error(format!(
                    "{field} must be a positive integer."
                )))
            }
        }
        Some(_) => Err(validation_error(format!(
            "{field} must be a finite number."
        ))),
    }
}

fn optional_input_mode(value: Option<&JsonValue>) -> Result<Option<InputMode>, ValidationError> {
    let Some(value) = optional_string(value, "source.input_mode")? else {
        return Ok(None);
    };
    match value.as_str() {
        "args" => Ok(Some(InputMode::Args)),
        "stdin" => Ok(Some(InputMode::Stdin)),
        "none" => Ok(Some(InputMode::None)),
        _ => Err(validation_error(
            "source.input_mode must be args, stdin, or none.",
        )),
    }
}

fn first_value<'a>(
    left: Option<&'a JsonValue>,
    right: Option<&'a JsonValue>,
) -> Option<&'a JsonValue> {
    match left {
        None | Some(JsonValue::Null) => right,
        Some(value) => Some(value),
    }
}

fn field_value<'a>(object: Option<&'a JsonObject>, field: &str) -> Option<&'a JsonValue> {
    object.and_then(|object| object.get(field))
}

fn nested_value<'a>(value: Option<&'a JsonValue>, field: &str) -> Option<&'a JsonValue> {
    match value {
        Some(JsonValue::Object(object)) => object.get(field),
        _ => None,
    }
}

fn default_agent_source() -> JsonObject {
    [("type".to_owned(), JsonValue::String("agent".to_owned()))]
        .into_iter()
        .collect()
}

fn validate_mcp_server(
    source: &JsonObject,
    source_type: &str,
) -> Result<Option<SkillMcpServer>, ValidationError> {
    if source_type != "mcp" {
        return Ok(None);
    }
    let server = required_object(source.get("server"), "source.server")?;
    Ok(Some(SkillMcpServer {
        command: required_string(server.get("command"), "source.server.command")?,
        args: optional_string_array(server.get("args"), "source.server.args")?.unwrap_or_default(),
        cwd: optional_string(server.get("cwd"), "source.server.cwd")?,
    }))
}

fn validate_mcp_tool(
    source: &JsonObject,
    source_type: &str,
) -> Result<Option<String>, ValidationError> {
    if source_type == "mcp" {
        return Ok(Some(required_string(source.get("tool"), "source.tool")?));
    }
    optional_string(source.get("tool"), "source.tool")
}

fn validate_catalog_ref(
    source: &JsonObject,
    source_type: &str,
) -> Result<Option<String>, ValidationError> {
    if source_type == "catalog" {
        return Ok(Some(required_string(
            source.get("catalog_ref"),
            "source.catalog_ref",
        )?));
    }
    optional_string(source.get("catalog_ref"), "source.catalog_ref")
}

fn validate_a2a_url(
    source: &JsonObject,
    source_type: &str,
) -> Result<Option<String>, ValidationError> {
    if source_type == "a2a" {
        return Ok(Some(required_string(
            source.get("agent_card_url"),
            "source.agent_card_url",
        )?));
    }
    optional_string(source.get("agent_card_url"), "source.agent_card_url")
}

fn validate_agent(
    source: &JsonObject,
    source_type: &str,
) -> Result<Option<String>, ValidationError> {
    if source_type == "agent-step" {
        return Ok(Some(required_string(source.get("agent"), "source.agent")?));
    }
    optional_string(source.get("agent"), "source.agent")
}

fn validate_task(
    source: &JsonObject,
    source_type: &str,
) -> Result<Option<String>, ValidationError> {
    if matches!(source_type, "agent-step" | "a2a") {
        return Ok(Some(required_string(source.get("task"), "source.task")?));
    }
    optional_string(source.get("task"), "source.task")
}

fn validate_hook(
    source: &JsonObject,
    source_type: &str,
) -> Result<Option<String>, ValidationError> {
    if source_type == "harness-hook" {
        return Ok(Some(required_string(source.get("hook"), "source.hook")?));
    }
    optional_string(source.get("hook"), "source.hook")
}

fn validate_graph_source(
    source: &JsonObject,
    source_type: &str,
) -> Result<Option<crate::ExecutionGraph>, ValidationError> {
    if source_type != "graph" {
        return Ok(None);
    }
    let graph = required_object(source.get("graph"), "source.graph")?.clone();
    validate_graph_document(graph.clone(), Some(RawGraphIr { document: graph })).map(Some)
}

fn validate_agent_command_boundary(
    source: &JsonObject,
    source_type: &str,
) -> Result<(), ValidationError> {
    if matches!(source_type, "agent-step" | "harness-hook")
        && (source.contains_key("command") || source.contains_key("args"))
    {
        return Err(validation_error(format!(
            "{source_type} sources must not declare source.command or source.args."
        )));
    }
    Ok(())
}

fn required_sandbox_profile(
    value: Option<&JsonValue>,
    field: &str,
) -> Result<String, ValidationError> {
    let profile = required_string(value, field)?;
    if matches!(
        profile.as_str(),
        "readonly" | "workspace-write" | "network" | "unrestricted-local-dev"
    ) {
        return Ok(profile);
    }
    Err(validation_error(format!(
        "{field} must be readonly, workspace-write, network, or unrestricted-local-dev."
    )))
}

fn optional_cwd_policy(value: Option<&JsonValue>) -> Result<Option<String>, ValidationError> {
    let Some(value) = optional_string(value, "sandbox.cwd_policy")? else {
        return Ok(None);
    };
    if matches!(value.as_str(), "skill-directory" | "workspace" | "custom") {
        return Ok(Some(value));
    }
    Err(validation_error(
        "sandbox.cwd_policy must be skill-directory, workspace, or custom.",
    ))
}

fn sandbox_declaration(
    profile: &str,
    cwd_policy: Option<&str>,
    env_allowlist: Option<Vec<String>>,
    network: Option<bool>,
    writable_paths: Option<Vec<String>>,
    require_enforcement: Option<bool>,
) -> Result<SandboxDeclaration, ValidationError> {
    Ok(SandboxDeclaration {
        profile: match profile {
            "readonly" => SandboxProfile::Readonly,
            "workspace-write" => SandboxProfile::WorkspaceWrite,
            "network" => SandboxProfile::Network,
            "unrestricted-local-dev" => SandboxProfile::UnrestrictedLocalDev,
            _ => return Err(validation_error("sandbox.profile is invalid.")),
        },
        cwd_policy: match cwd_policy {
            None => None,
            Some("skill-directory") => Some(CwdPolicy::SkillDirectory),
            Some("workspace") => Some(CwdPolicy::Workspace),
            Some("custom") => Some(CwdPolicy::Custom),
            Some(_) => return Err(validation_error("sandbox.cwd_policy is invalid.")),
        },
        env_allowlist,
        network,
        writable_paths,
        require_enforcement,
    })
}

fn validate_outcome(
    value: Option<&JsonValue>,
    field: &str,
) -> Result<Option<ReceiptOutcome>, ValidationError> {
    let Some(record) = optional_object(value, field)? else {
        return Ok(None);
    };
    Ok(Some(ReceiptOutcome {
        code: optional_string(record.get("code"), &format!("{field}.code"))?,
        summary: optional_string(record.get("summary"), &format!("{field}.summary"))?,
        observed_at: optional_string(record.get("observed_at"), &format!("{field}.observed_at"))?,
        data: optional_object(record.get("data"), &format!("{field}.data"))?,
    }))
}

fn validate_input_context(
    value: Option<&JsonValue>,
    field: &str,
) -> Result<Option<InputContextCapture>, ValidationError> {
    let Some(record) = optional_object(value, field)? else {
        return Ok(None);
    };
    let max_bytes = optional_u64(record.get("max_bytes"), &format!("{field}.max_bytes"))?;
    if matches!(max_bytes, Some(0)) {
        return Err(validation_error(format!(
            "{field}.max_bytes must be a positive integer."
        )));
    }
    Ok(Some(InputContextCapture {
        capture: optional_bool(record.get("capture"), &format!("{field}.capture"))?,
        source: optional_string(record.get("source"), &format!("{field}.source"))?,
        max_bytes,
        snapshot: record.get("snapshot").cloned(),
    }))
}

fn validate_surface_refs(
    value: Option<&JsonValue>,
    field: &str,
) -> Result<Option<Vec<ReceiptSurfaceRef>>, ValidationError> {
    let Some(values) = optional_array(value, field)? else {
        return Ok(None);
    };
    values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let record = required_object(Some(value), &format!("{field}[{index}]"))?;
            Ok(ReceiptSurfaceRef {
                surface_type: required_string(
                    record.get("type"),
                    &format!("{field}[{index}].type"),
                )?,
                uri: required_string(record.get("uri"), &format!("{field}[{index}].uri"))?,
                label: optional_string(record.get("label"), &format!("{field}[{index}].label"))?,
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Some)
}

fn optional_array<'a>(
    value: Option<&'a JsonValue>,
    field: &str,
) -> Result<Option<&'a [JsonValue]>, ValidationError> {
    match value {
        None | Some(JsonValue::Null) => Ok(None),
        Some(JsonValue::Array(values)) => Ok(Some(values)),
        Some(_) => Err(validation_error(format!(
            "{field} must be an array when present."
        ))),
    }
}

fn optional_disposition(
    value: Option<&JsonValue>,
    field: &str,
) -> Result<Option<GovernedDisposition>, ValidationError> {
    match optional_string(value, field)?.as_deref() {
        None => Ok(None),
        Some("completed") => Ok(Some(GovernedDisposition::Completed)),
        Some("needs_agent") => Ok(Some(GovernedDisposition::NeedsAgent)),
        Some("policy_denied") => Ok(Some(GovernedDisposition::PolicyDenied)),
        Some("approval_required") => Ok(Some(GovernedDisposition::ApprovalRequired)),
        Some("observing") => Ok(Some(GovernedDisposition::Observing)),
        Some("escalated") => Ok(Some(GovernedDisposition::Escalated)),
        Some(_) => Err(validation_error(format!(
            "{field} must be one of completed, needs_agent, policy_denied, approval_required, observing, escalated."
        ))),
    }
}

fn optional_outcome_state(
    value: Option<&JsonValue>,
    field: &str,
) -> Result<Option<OutcomeState>, ValidationError> {
    match optional_string(value, field)?.as_deref() {
        None => Ok(None),
        Some("pending") => Ok(Some(OutcomeState::Pending)),
        Some("complete") => Ok(Some(OutcomeState::Complete)),
        Some("expired") => Ok(Some(OutcomeState::Expired)),
        Some(_) => Err(validation_error(format!(
            "{field} must be one of pending, complete, or expired."
        ))),
    }
}

fn validate_named_emits(
    value: Option<&JsonValue>,
    field: &str,
) -> Result<Option<BTreeMap<String, String>>, ValidationError> {
    let Some(record) = optional_object(value, field)? else {
        return Ok(None);
    };
    record
        .into_iter()
        .map(|(key, value)| {
            let JsonValue::String(value) = value else {
                return Err(validation_error(format!(
                    "{field}.{key} must be a non-empty string."
                )));
            };
            if value.trim().is_empty() {
                return Err(validation_error(format!(
                    "{field}.{key} must be a non-empty string."
                )));
            }
            Ok((key, value))
        })
        .collect::<Result<BTreeMap<_, _>, _>>()
        .map(Some)
}

fn validate_allowed_tools(
    value: Option<&JsonValue>,
    field: &str,
) -> Result<Option<Vec<String>>, ValidationError> {
    let Some(values) = optional_string_array(value, field)? else {
        return Ok(None);
    };
    for value in &values {
        if value.trim().is_empty() {
            return Err(validation_error(format!(
                "{field} entries must not be empty."
            )));
        }
    }
    Ok(Some(values))
}
