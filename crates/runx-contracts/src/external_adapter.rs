//! External adapter contract types.
use serde::{Deserialize, Deserializer, Serialize};

use crate::{JsonNumber, JsonObject, Reference, ResolutionRequest};

pub const EXTERNAL_ADAPTER_PROTOCOL_VERSION: &str = "runx.external_adapter.v1";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExternalAdapterTransportKind {
    Process,
    Http,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExternalAdapterStatus {
    Completed,
    Failed,
    HostResolutionRequested,
    Cancelled,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExternalAdapterCredentialPurpose {
    ProviderApi,
    Registry,
    ArtifactStore,
    WebhookVerification,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExternalAdapterTransport {
    pub kind: ExternalAdapterTransportKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExternalAdapterCredentialNeed {
    pub purpose: ExternalAdapterCredentialPurpose,
    pub provider: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope_refs: Option<Vec<Reference>>,
    pub required: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExternalAdapterSandboxIntent {
    pub profile: String,
    pub network: bool,
    pub cwd_policy: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub writable_paths: Option<Vec<String>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExternalAdapterTimeouts {
    pub startup_ms: u64,
    pub invocation_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExternalAdapterManifest {
    pub schema: String,
    pub protocol_version: String,
    pub adapter_id: String,
    pub name: String,
    pub version: String,
    pub supported_source_types: Vec<String>,
    pub transport: ExternalAdapterTransport,
    pub timeouts: ExternalAdapterTimeouts,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential_needs: Option<Vec<ExternalAdapterCredentialNeed>>,
    pub sandbox_intent: ExternalAdapterSandboxIntent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<JsonObject>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExternalAdapterCredentialReference {
    pub credential_ref: Reference,
    pub provider: String,
    pub purpose: ExternalAdapterCredentialPurpose,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExternalAdapterCredentialRequest {
    pub schema: String,
    pub protocol_version: String,
    pub request_id: String,
    pub adapter_id: String,
    pub invocation_id: String,
    pub credential_refs: Vec<ExternalAdapterCredentialReference>,
    pub requested_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExternalAdapterInvocation {
    pub schema: String,
    pub protocol_version: String,
    pub invocation_id: String,
    pub adapter_id: String,
    pub run_id: String,
    pub step_id: String,
    pub source_type: String,
    pub skill_ref: String,
    pub harness_ref: Reference,
    pub host_ref: Reference,
    pub inputs: JsonObject,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_inputs: Option<JsonObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receipt_dir: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<JsonObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential_refs: Option<Vec<ExternalAdapterCredentialReference>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<JsonObject>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExternalAdapterArtifactObservation {
    pub artifact_ref: Reference,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExternalAdapterErrorObservation {
    pub code: String,
    pub message: String,
    pub retryable: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ExternalAdapterTelemetryValue {
    Number(JsonNumber),
    String(String),
    Bool(bool),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExternalAdapterTelemetryObservation {
    pub name: String,
    pub value: ExternalAdapterTelemetryValue,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExternalAdapterResponse {
    pub schema: String,
    pub protocol_version: String,
    pub invocation_id: String,
    pub adapter_id: String,
    pub status: ExternalAdapterStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdout: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stderr: Option<String>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_nullable_i64",
        skip_serializing_if = "Option::is_none"
    )]
    pub exit_code: Option<Option<i64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<JsonObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifacts: Option<Vec<ExternalAdapterArtifactObservation>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<ExternalAdapterErrorObservation>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub telemetry: Option<Vec<ExternalAdapterTelemetryObservation>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<JsonObject>,
    pub observed_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExternalAdapterHostResolutionFrame {
    pub schema: String,
    pub protocol_version: String,
    pub frame_id: String,
    pub invocation_id: String,
    pub adapter_id: String,
    pub request: ResolutionRequest,
    pub requested_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExternalAdapterCancellationFrame {
    pub schema: String,
    pub protocol_version: String,
    pub frame_id: String,
    pub invocation_id: String,
    pub adapter_id: String,
    pub reason: String,
    pub requested_at: String,
}

fn deserialize_optional_nullable_i64<'de, D>(
    deserializer: D,
) -> Result<Option<Option<i64>>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<i64>::deserialize(deserializer).map(Some)
}
