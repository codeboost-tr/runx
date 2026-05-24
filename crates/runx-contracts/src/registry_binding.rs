//! Upstream registry binding contract (`runx.registry_binding.v1`): the open
//! (`additionalProperties: true`) document tying a skill to its upstream source,
//! registry placement, and harness verification status.
//!
//! Identity is the legacy bare `runx.ai/schemas` `$id` (no `x-runx-schema`).
use serde::{Deserialize, Serialize};

use crate::schema::RunxSchema;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
pub enum RegistryBindingSchema {
    #[serde(rename = "runx.registry_binding.v1")]
    V1,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(rename_all = "snake_case")]
pub enum RegistryBindingState {
    RegistryBindingDrafted,
    RegistryBound,
    HarnessVerified,
    Published,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(rename_all = "snake_case")]
pub enum RegistryTrustTier {
    FirstParty,
    Verified,
    Community,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(rename_all = "snake_case")]
pub enum RegistryHarnessStatus {
    Pending,
    Failed,
    HarnessVerified,
}

/// The skill identity for a registry binding. Open (`additionalProperties:
/// true`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
pub struct RegistryBindingSkill {
    pub id: String,
    pub name: String,
    pub description: String,
}

/// The upstream source of truth for a registry binding. Open.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
pub struct RegistryBindingUpstream {
    pub host: String,
    pub owner: String,
    pub repo: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    pub commit: String,
    pub blob_sha: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pr_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merged_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_url: Option<String>,
    /// Committed as `const: true`; modeled as a boolean (the const bound is not
    /// expressible by the emitter and is exercised only at the `true` value).
    pub source_of_truth: bool,
}

/// The registry placement for a binding. Open.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
pub struct RegistryBindingRegistry {
    pub owner: String,
    pub trust_tier: RegistryTrustTier,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install_command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_command: Option<String>,
    pub profile_path: String,
    /// Committed as `const: true`; see [`RegistryBindingUpstream::source_of_truth`].
    pub materialized_package_is_registry_artifact: bool,
}

/// The harness verification status for a binding. Open.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, RunxSchema)]
pub struct RegistryBindingHarness {
    pub status: RegistryHarnessStatus,
    pub case_count: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assertion_count: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub case_names: Option<Vec<String>>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, RunxSchema)]
#[runx_schema(spec_id = "https://runx.ai/schemas/registry-binding.schema.json")]
pub struct RegistryBinding {
    pub schema: RegistryBindingSchema,
    pub state: RegistryBindingState,
    pub skill: RegistryBindingSkill,
    pub upstream: RegistryBindingUpstream,
    pub registry: RegistryBindingRegistry,
    pub harness: RegistryBindingHarness,
}
