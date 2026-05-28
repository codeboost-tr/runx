//! Command, observation, and projection shapes for the target-repo runner
//! lifecycle. Pure data; the orchestration that produces these values lives in
//! the parent module's execution / pull_request / revision / source_publication
//! / projection slices.

use serde::Serialize;

use runx_contracts::{
    JsonObject, Receipt, Reference, TargetRepoRunnerDedupeLookupExecution,
    TargetRepoRunnerDedupeLookupObservation, TargetRepoRunnerExecutionPlan,
    TargetRepoRunnerExistingPullRequest, TargetRepoRunnerPlan, TargetRepoRunnerProvider,
    TargetRepoRunnerPullRequestDisposition, TargetRepoRunnerPullRequestReceiptPlan,
    TargetRepoRunnerReadinessObservation, TargetRepoRunnerSourcePublicationReceiptPlan,
};

use super::provider::{
    TargetRepoRunnerGithubPullRequestSearchCommand, TargetRepoRunnerGithubRepository,
};

#[derive(Clone, Debug, PartialEq)]
pub struct TargetRepoRunnerFixtureExecutionInput {
    pub plan: TargetRepoRunnerPlan,
    pub readiness: TargetRepoRunnerReadinessObservation,
    pub dedupe: TargetRepoRunnerDedupeLookupObservation,
    pub created_pull_request: Option<TargetRepoRunnerExistingPullRequest>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TargetRepoRunnerFixtureExecution {
    pub execution_plan: TargetRepoRunnerExecutionPlan,
    pub dedupe_execution: TargetRepoRunnerDedupeLookupExecution,
    pub deduped_plan: TargetRepoRunnerPlan,
    pub disposition: TargetRepoRunnerPullRequestDisposition,
    pub pull_request: TargetRepoRunnerExistingPullRequest,
    pub pull_request_receipt: TargetRepoRunnerPullRequestReceiptPlan,
    pub source_publication_receipt: TargetRepoRunnerSourcePublicationReceiptPlan,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TargetRepoRunnerLiveExecution {
    pub checkout_command: TargetRepoRunnerCheckoutCommand,
    pub readiness: TargetRepoRunnerReadinessObservation,
    pub provider_lookup_command: TargetRepoRunnerProviderDedupeLookupCommand,
    pub dedupe_observation: TargetRepoRunnerDedupeLookupObservation,
    pub runner_observation: Option<TargetRepoRunnerGovernedRunnerObservation>,
    pub git_mutation_command: Option<TargetRepoRunnerGitMutationCommand>,
    pub git_mutation_observation: Option<TargetRepoRunnerGitMutationObservation>,
    pub pull_request_request: TargetRepoRunnerPullRequestObservationRequest,
    pub pull_request_observation: TargetRepoRunnerPullRequestObservation,
    pub execution: TargetRepoRunnerFixtureExecution,
    pub revision_receipt: Receipt,
    pub revision_projection: TargetRepoRunnerRevisionReceiptProjection,
    pub source_publication_request: TargetRepoRunnerSourcePublicationRequest,
    pub source_publication_observation: TargetRepoRunnerSourcePublicationObservation,
    pub source_publication_receipt: Receipt,
    pub source_publication_projection: TargetRepoRunnerSourcePublicationProjection,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TargetRepoRunnerGovernedRunnerInvocation {
    pub execution_plan: TargetRepoRunnerExecutionPlan,
    pub deduped_plan: TargetRepoRunnerPlan,
    pub disposition: TargetRepoRunnerPullRequestDisposition,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TargetRepoRunnerGovernedRunnerObservation {
    pub runner_id: String,
    pub target_repo: String,
    pub summary: String,
    pub revision_refs: Vec<Reference>,
    pub artifact_refs: Vec<Reference>,
    pub verification_refs: Vec<Reference>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TargetRepoRunnerGitMutationCommand {
    pub provider: TargetRepoRunnerProvider,
    pub target_repo: String,
    pub repository: TargetRepoRunnerGithubRepository,
    pub target_repo_ref: Reference,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_branch: Option<String>,
    pub branch: String,
    pub dedupe_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_issue_ref: Option<Reference>,
    pub source_thread_ref: Reference,
    pub runner_id: String,
    pub runner_summary: String,
    pub runner_revision_refs: Vec<Reference>,
    pub artifact_refs: Vec<Reference>,
    pub verification_refs: Vec<Reference>,
    pub human_merge_gate_required: bool,
    pub local_path_hidden: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TargetRepoRunnerGitMutationObservation {
    pub target_repo: String,
    pub branch: String,
    pub head_sha: String,
    pub revision_refs: Vec<Reference>,
    pub verification_refs: Vec<Reference>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TargetRepoRunnerCheckoutCommand {
    pub target_repo: String,
    pub public_repo_ref: Reference,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_branch: Option<String>,
    pub runner_id: String,
    pub runner_kind: runx_contracts::schema::NonEmptyString,
    pub target_scafld_required: bool,
    pub runner_scafld_required: bool,
    pub mutate_target_repo: bool,
    pub local_path_hidden: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TargetRepoRunnerProviderDedupeLookupCommand {
    pub provider: TargetRepoRunnerProvider,
    pub target_repo: String,
    pub repository: TargetRepoRunnerGithubRepository,
    pub dedupe_key: String,
    pub result_limit: u16,
    pub query: TargetRepoRunnerGithubPullRequestSearchCommand,
    pub markers: Vec<String>,
    pub required_refs: Vec<Reference>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TargetRepoRunnerPullRequestObservationRequest {
    pub command: TargetRepoRunnerPullRequestMutationCommand,
    pub disposition: TargetRepoRunnerPullRequestDisposition,
    pub target_repo: String,
    pub dedupe_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub existing_pull_request: Option<TargetRepoRunnerExistingPullRequest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runner_observation: Option<TargetRepoRunnerGovernedRunnerObservation>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TargetRepoRunnerPullRequestMutationCommand {
    pub provider: TargetRepoRunnerProvider,
    pub disposition: TargetRepoRunnerPullRequestDisposition,
    pub target_repo: String,
    pub repository: TargetRepoRunnerGithubRepository,
    pub target_repo_ref: Reference,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_branch: Option<String>,
    pub dedupe_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_issue_ref: Option<Reference>,
    pub source_thread_ref: Reference,
    pub mutation: TargetRepoRunnerPullRequestMutation,
    pub human_merge_gate_required: bool,
    pub local_path_hidden: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TargetRepoRunnerPullRequestMutation {
    Create(TargetRepoRunnerPullRequestCreateCommand),
    Reuse(TargetRepoRunnerPullRequestReuseCommand),
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TargetRepoRunnerPullRequestCreateCommand {
    pub title: String,
    pub body: String,
    pub head_branch: String,
    pub head_sha: String,
    pub runner_id: String,
    pub runner_summary: String,
    pub runner_revision_refs: Vec<Reference>,
    pub git_revision_refs: Vec<Reference>,
    pub artifact_refs: Vec<Reference>,
    pub verification_refs: Vec<Reference>,
    pub git_verification_refs: Vec<Reference>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TargetRepoRunnerPullRequestReuseCommand {
    pub existing_pull_request: TargetRepoRunnerExistingPullRequest,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TargetRepoRunnerPullRequestObservation {
    pub provider: TargetRepoRunnerProvider,
    pub target_repo: String,
    pub pull_request: TargetRepoRunnerExistingPullRequest,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub head_branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub head_sha: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TargetRepoRunnerSourcePublicationRequest {
    pub publication: TargetRepoRunnerSourcePublicationReceiptPlan,
    pub revision_receipt_ref: Reference,
    pub revision_projection: TargetRepoRunnerRevisionReceiptProjection,
    pub commands: Vec<TargetRepoRunnerSourcePublicationCommand>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TargetRepoRunnerSourcePublicationCommand {
    SourceIssueComment { target: Reference, body: String },
    SourceThreadReply { target: Reference, body: String },
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TargetRepoRunnerSourcePublicationObservation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_issue_ref: Option<Reference>,
    pub source_thread_ref: Reference,
    pub pull_request_ref: Reference,
    pub revision_receipt_ref: Reference,
    pub published_refs: Vec<Reference>,
    pub metadata: JsonObject,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TargetRepoRunnerRevisionReceiptProjection {
    pub receipt_ref: Reference,
    pub act_id: String,
    pub disposition: TargetRepoRunnerPullRequestDisposition,
    pub target_repo_ref: Reference,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_issue_ref: Option<Reference>,
    pub source_thread_ref: Reference,
    pub pull_request_ref: Reference,
    pub summary: String,
    pub metadata: JsonObject,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TargetRepoRunnerSourcePublicationProjection {
    pub receipt_ref: Reference,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_issue_ref: Option<Reference>,
    pub source_thread_ref: Reference,
    pub pull_request_ref: Reference,
    pub published_refs: Vec<Reference>,
    pub summary: String,
    pub metadata: JsonObject,
}
