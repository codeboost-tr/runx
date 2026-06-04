//! First-class graph-step adapter for the thread-outbox provider protocol.
//!
//! The process supervisor owns provider publication and readback validation.
//! This adapter is deliberately small: it resolves skill-local manifest/request
//! frames, invokes the supervisor, and projects the accepted observation into the
//! universal graph-step output shape.

use std::path::{Component, Path, PathBuf};

use runx_contracts::{
    JsonObject, JsonValue, ThreadOutboxProviderFetch, ThreadOutboxProviderManifest,
    ThreadOutboxProviderOperation, ThreadOutboxProviderPush,
};
use serde::de::DeserializeOwned;
use thiserror::Error;

use crate::RuntimeError;
use crate::adapter::{
    CREDENTIAL_DELIVERY_OBSERVATIONS_METADATA, InvocationStatus, SkillAdapter, SkillInvocation,
    SkillOutput,
};
use crate::outbox_provider::{
    ThreadOutboxProviderProcessOutcome, ThreadOutboxProviderProcessSupervisor,
    ThreadOutboxProviderSupervisorError, ThreadOutboxProviderSupervisorOptions,
};

const THREAD_OUTBOX_PROVIDER: &str = "thread-outbox-provider";
const CONFIG_FIELD: &str = "thread_outbox_provider";
const MANIFEST_PATH_FIELD: &str = "manifest_path";
const OPERATION_FIELD: &str = "operation";
const PUSH_PATH_FIELD: &str = "push_path";
const FETCH_PATH_FIELD: &str = "fetch_path";
const OBSERVATION_METADATA: &str = "thread_outbox_provider_observation";
const OPERATION_METADATA: &str = "thread_outbox_provider_operation";
const PROVIDER_LOCATOR_METADATA: &str = "thread_outbox_provider_locator";
const PROVIDER_EVENT_HASH_METADATA: &str = "thread_outbox_provider_event_hash";

#[derive(Clone, Debug, Default)]
pub struct ThreadOutboxProviderSkillAdapter {
    supervisor_options: ThreadOutboxProviderSupervisorOptions,
}

impl SkillAdapter for ThreadOutboxProviderSkillAdapter {
    fn adapter_type(&self) -> &'static str {
        THREAD_OUTBOX_PROVIDER
    }

    fn invoke(&self, request: SkillInvocation) -> Result<SkillOutput, RuntimeError> {
        if request.source.source_type != runx_parser::SourceKind::ThreadOutboxProvider {
            return Err(RuntimeError::UnsupportedAdapter {
                adapter_type: request.source.source_type.as_str().to_owned(),
            });
        }
        let skill_name = request.skill_name.clone();
        invoke_thread_outbox_provider_skill(request, &self.supervisor_options).map_err(|error| {
            RuntimeError::SkillFailed {
                skill_name,
                message: error.to_string(),
            }
        })
    }
}

#[derive(Debug, Error)]
pub enum ThreadOutboxProviderSkillAdapterError {
    #[error("thread-outbox-provider source is missing source.thread_outbox_provider")]
    MissingConfig,
    #[error("thread-outbox-provider source.thread_outbox_provider must be an object")]
    InvalidConfigShape,
    #[error("thread-outbox-provider source.thread_outbox_provider.{field} is required")]
    MissingConfigField { field: &'static str },
    #[error("thread-outbox-provider source.thread_outbox_provider.{field} must be a string")]
    InvalidConfigField { field: &'static str },
    #[error("thread-outbox-provider operation must be push or fetch, got '{operation}'")]
    InvalidOperation { operation: String },
    #[error(
        "thread-outbox-provider {field} must be a relative path below the skill directory: '{path}'"
    )]
    InvalidFramePath { field: &'static str, path: String },
    #[error(
        "thread-outbox-provider {field} '{path}' escapes the skill directory '{skill_directory}'"
    )]
    FramePathEscapesSkillDirectory {
        field: &'static str,
        path: String,
        skill_directory: String,
    },
    #[error("thread-outbox-provider {field} file '{path}' could not be read: {source}")]
    FrameRead {
        field: &'static str,
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("thread-outbox-provider JSON failed while {context}: {source}")]
    Json {
        context: String,
        #[source]
        source: serde_json::Error,
    },
    #[error(transparent)]
    Supervisor(#[from] ThreadOutboxProviderSupervisorError),
}

#[derive(Clone, Debug)]
struct ThreadOutboxProviderConfig {
    manifest_path: String,
    operation: ThreadOutboxProviderOperation,
    push_path: Option<String>,
    fetch_path: Option<String>,
}

fn invoke_thread_outbox_provider_skill(
    request: SkillInvocation,
    supervisor_options: &ThreadOutboxProviderSupervisorOptions,
) -> Result<SkillOutput, ThreadOutboxProviderSkillAdapterError> {
    let config = config_from_source(&request.source.raw)?;
    let manifest: ThreadOutboxProviderManifest = contract_from_skill_file(
        &request.skill_directory,
        MANIFEST_PATH_FIELD,
        &config.manifest_path,
    )?;
    let supervisor =
        ThreadOutboxProviderProcessSupervisor::new(ThreadOutboxProviderSupervisorOptions {
            cwd: Some(canonical_skill_directory(
                &request.skill_directory,
                MANIFEST_PATH_FIELD,
            )?),
            ..supervisor_options.clone()
        });
    let outcome = match config.operation {
        ThreadOutboxProviderOperation::Push => {
            let push_path = config.push_path.as_deref().ok_or(
                ThreadOutboxProviderSkillAdapterError::MissingConfigField {
                    field: PUSH_PATH_FIELD,
                },
            )?;
            let push: ThreadOutboxProviderPush =
                contract_from_skill_file(&request.skill_directory, PUSH_PATH_FIELD, push_path)?;
            supervisor.invoke_push(&manifest, &push, &request.credential_delivery)?
        }
        ThreadOutboxProviderOperation::Fetch => {
            let fetch_path = config.fetch_path.as_deref().ok_or(
                ThreadOutboxProviderSkillAdapterError::MissingConfigField {
                    field: FETCH_PATH_FIELD,
                },
            )?;
            let fetch: ThreadOutboxProviderFetch =
                contract_from_skill_file(&request.skill_directory, FETCH_PATH_FIELD, fetch_path)?;
            supervisor.invoke_fetch(&manifest, &fetch, &request.credential_delivery)?
        }
    };
    skill_output_from_outcome(outcome)
}

fn config_from_source(
    source: &JsonObject,
) -> Result<ThreadOutboxProviderConfig, ThreadOutboxProviderSkillAdapterError> {
    let config = match source.get(CONFIG_FIELD) {
        Some(JsonValue::Object(config)) => config,
        Some(_) => return Err(ThreadOutboxProviderSkillAdapterError::InvalidConfigShape),
        None => return Err(ThreadOutboxProviderSkillAdapterError::MissingConfig),
    };
    let manifest_path = required_config_string(config, MANIFEST_PATH_FIELD)?;
    let operation_raw = required_config_string(config, OPERATION_FIELD)?;
    let operation = match operation_raw.as_str() {
        "push" => ThreadOutboxProviderOperation::Push,
        "fetch" => ThreadOutboxProviderOperation::Fetch,
        other => {
            return Err(ThreadOutboxProviderSkillAdapterError::InvalidOperation {
                operation: other.to_owned(),
            });
        }
    };
    Ok(ThreadOutboxProviderConfig {
        manifest_path,
        operation,
        push_path: optional_config_string(config, PUSH_PATH_FIELD)?,
        fetch_path: optional_config_string(config, FETCH_PATH_FIELD)?,
    })
}

fn required_config_string(
    config: &JsonObject,
    field: &'static str,
) -> Result<String, ThreadOutboxProviderSkillAdapterError> {
    optional_config_string(config, field)?
        .ok_or(ThreadOutboxProviderSkillAdapterError::MissingConfigField { field })
}

fn optional_config_string(
    config: &JsonObject,
    field: &'static str,
) -> Result<Option<String>, ThreadOutboxProviderSkillAdapterError> {
    match config.get(field) {
        Some(JsonValue::String(value)) => Ok(Some(value.clone())),
        Some(_) => Err(ThreadOutboxProviderSkillAdapterError::InvalidConfigField { field }),
        None => Ok(None),
    }
}

fn contract_from_skill_file<T>(
    skill_directory: &Path,
    field: &'static str,
    relative_path: &str,
) -> Result<T, ThreadOutboxProviderSkillAdapterError>
where
    T: DeserializeOwned,
{
    let path = skill_file_path(skill_directory, field, relative_path)?;
    let bytes = std::fs::read(&path).map_err(|source| {
        ThreadOutboxProviderSkillAdapterError::FrameRead {
            field,
            path: path.to_string_lossy().into_owned(),
            source,
        }
    })?;
    let value: JsonValue = serde_json::from_slice(&bytes).map_err(|source| {
        json_error(
            format!("parsing thread-outbox-provider {field} file"),
            source,
        )
    })?;
    let value = match value {
        JsonValue::Object(mut object) => {
            let expected = object.remove("expected");
            expected.unwrap_or(JsonValue::Object(object))
        }
        other => other,
    };
    let value = serde_json::to_value(&value).map_err(|source| {
        json_error(
            format!("serializing thread-outbox-provider {field} frame"),
            source,
        )
    })?;
    serde_json::from_value(value).map_err(|source| {
        json_error(
            format!("validating thread-outbox-provider {field} frame"),
            source,
        )
    })
}

fn skill_file_path(
    skill_directory: &Path,
    field: &'static str,
    relative_path: &str,
) -> Result<PathBuf, ThreadOutboxProviderSkillAdapterError> {
    validate_relative_path(field, relative_path)?;
    let skill_directory_display = skill_directory.to_string_lossy().into_owned();
    let skill_directory = canonical_skill_directory(skill_directory, field)?;
    let path = skill_directory.join(relative_path);
    let canonical_path =
        path.canonicalize()
            .map_err(|source| ThreadOutboxProviderSkillAdapterError::FrameRead {
                field,
                path: path.to_string_lossy().into_owned(),
                source,
            })?;
    if !canonical_path.starts_with(&skill_directory) {
        return Err(
            ThreadOutboxProviderSkillAdapterError::FramePathEscapesSkillDirectory {
                field,
                path: relative_path.to_owned(),
                skill_directory: skill_directory_display,
            },
        );
    }
    Ok(canonical_path)
}

fn canonical_skill_directory(
    skill_directory: &Path,
    field: &'static str,
) -> Result<PathBuf, ThreadOutboxProviderSkillAdapterError> {
    skill_directory.canonicalize().map_err(|source| {
        ThreadOutboxProviderSkillAdapterError::FrameRead {
            field,
            path: skill_directory.to_string_lossy().into_owned(),
            source,
        }
    })
}

fn validate_relative_path(
    field: &'static str,
    relative_path: &str,
) -> Result<(), ThreadOutboxProviderSkillAdapterError> {
    let path = Path::new(relative_path);
    let valid = !relative_path.trim().is_empty()
        && path.is_relative()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)));
    if valid {
        Ok(())
    } else {
        Err(ThreadOutboxProviderSkillAdapterError::InvalidFramePath {
            field,
            path: relative_path.to_owned(),
        })
    }
}

fn skill_output_from_outcome(
    outcome: ThreadOutboxProviderProcessOutcome,
) -> Result<SkillOutput, ThreadOutboxProviderSkillAdapterError> {
    let observation_value = contract_json_value(&outcome.observation, "serializing observation")?;
    let stdout = serde_json::to_string(&outcome.observation).map_err(|source| {
        json_error(
            "serializing thread-outbox-provider observation stdout",
            source,
        )
    })?;
    let mut metadata = JsonObject::new();
    metadata.insert(OBSERVATION_METADATA.to_owned(), observation_value);
    metadata.insert(
        OPERATION_METADATA.to_owned(),
        JsonValue::String(operation_label(&outcome.observation.operation).to_owned()),
    );
    if let Some(locator) = &outcome.observation.provider_locator {
        metadata.insert(
            PROVIDER_LOCATOR_METADATA.to_owned(),
            JsonValue::String(locator.locator.to_string()),
        );
    }
    if let Some(event_hash) = &outcome.observation.provider_event_id_hash {
        metadata.insert(
            PROVIDER_EVENT_HASH_METADATA.to_owned(),
            JsonValue::String(event_hash.to_string()),
        );
    }
    if let Some(delivery_observations) = &outcome.observation.delivery_observations {
        metadata.insert(
            CREDENTIAL_DELIVERY_OBSERVATIONS_METADATA.to_owned(),
            contract_json_value(delivery_observations, "serializing delivery observations")?,
        );
    }
    Ok(SkillOutput {
        status: InvocationStatus::Success,
        stdout,
        stderr: outcome.redacted_stderr,
        exit_code: outcome.process_exit_code,
        duration_ms: outcome.duration_ms,
        metadata,
    })
}

fn operation_label(operation: &ThreadOutboxProviderOperation) -> &'static str {
    match operation {
        ThreadOutboxProviderOperation::Push => "push",
        ThreadOutboxProviderOperation::Fetch => "fetch",
    }
}

fn contract_json_value(
    value: &impl serde::Serialize,
    context: &'static str,
) -> Result<JsonValue, ThreadOutboxProviderSkillAdapterError> {
    let value = serde_json::to_value(value).map_err(|source| json_error(context, source))?;
    serde_json::from_value(value).map_err(|source| json_error(context, source))
}

fn json_error(
    context: impl Into<String>,
    source: serde_json::Error,
) -> ThreadOutboxProviderSkillAdapterError {
    ThreadOutboxProviderSkillAdapterError::Json {
        context: context.into(),
        source,
    }
}
