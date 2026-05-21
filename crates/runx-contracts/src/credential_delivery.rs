//! Credential delivery contracts: public refs, handles, and observations only.
use serde::{Deserialize, Serialize};

use crate::Reference;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialDeliveryMode {
    ProcessEnv,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialDeliveryPurpose {
    ProviderApi,
    Registry,
    ArtifactStore,
    WebhookVerification,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialMaterialRole {
    AccessToken,
    RefreshToken,
    ApiKey,
    ClientSecret,
    SessionToken,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialDeliveryStatus {
    Delivered,
    Denied,
    NotFound,
    ProfileMismatch,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialDeliveryObservationStatus {
    Delivered,
    Denied,
    NotDelivered,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CredentialDeliveryEnvBinding {
    pub role: CredentialMaterialRole,
    pub env_var: String,
    pub required: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CredentialDeliveryProfile {
    pub schema: String,
    pub profile_id: String,
    pub provider: String,
    pub auth_mode: String,
    pub purpose: CredentialDeliveryPurpose,
    pub delivery_mode: CredentialDeliveryMode,
    pub material_roles: Vec<CredentialMaterialRole>,
    pub env_bindings: Vec<CredentialDeliveryEnvBinding>,
    pub redaction_policy_ref: Reference,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CredentialDeliveryRequest {
    pub schema: String,
    pub request_id: String,
    pub harness_ref: Reference,
    pub host_ref: Reference,
    pub grant_ref: Reference,
    pub credential_ref: Reference,
    pub profile_id: String,
    pub provider: String,
    pub purpose: CredentialDeliveryPurpose,
    pub requested_roles: Vec<CredentialMaterialRole>,
    pub requested_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CredentialDeliveryHandle {
    pub role: CredentialMaterialRole,
    pub delivery_handle_ref: Reference,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env_var: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CredentialDeliveryBrokerResponse {
    pub schema: String,
    pub response_id: String,
    pub request_id: String,
    pub status: CredentialDeliveryStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delivery_mode: Option<CredentialDeliveryMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handles: Option<Vec<CredentialDeliveryHandle>>,
    pub credential_refs: Vec<Reference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub material_ref_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub denied_reasons: Option<Vec<String>>,
    pub issued_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CredentialDeliveryObservation {
    pub schema: String,
    pub observation_id: String,
    pub request_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_id: Option<String>,
    pub status: CredentialDeliveryObservationStatus,
    pub harness_ref: Reference,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_ref: Option<Reference>,
    pub profile_id: String,
    pub provider: String,
    pub purpose: CredentialDeliveryPurpose,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delivery_mode: Option<CredentialDeliveryMode>,
    pub credential_refs: Vec<Reference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub material_ref_hash: Option<String>,
    pub delivered_roles: Vec<CredentialMaterialRole>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redaction_refs: Option<Vec<Reference>>,
    pub observed_at: String,
}
