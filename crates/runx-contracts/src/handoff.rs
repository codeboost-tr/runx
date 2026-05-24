//! Handoff boundary contracts: per-signal records (`runx.handoff_signal.v1`)
//! and the rolled-up handoff state (`runx.handoff_state.v1`).
use serde::{Deserialize, Serialize};

use crate::schema::{IsoDateTime, NonEmptyString, RunxSchema};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
pub enum HandoffSignalSchema {
    #[serde(rename = "runx.handoff_signal.v1")]
    V1,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandoffSignalSource {
    PullRequestComment,
    PullRequestReview,
    PullRequestState,
    IssueComment,
    DiscussionReply,
    EmailReply,
    DirectMessageReply,
    ManualNote,
    SystemEvent,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandoffDisposition {
    Acknowledged,
    Interested,
    RequestedChanges,
    Accepted,
    ApprovedToSend,
    Merged,
    Declined,
    RequestedNoContact,
    Rerouted,
}

/// The actor attribution carried on a handoff signal.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(deny_unknown_fields)]
pub struct HandoffSignalActor {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor_id: Option<NonEmptyString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<NonEmptyString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_identity: Option<NonEmptyString>,
}

/// The handoff signal's own source reference (a distinct, smaller shape than the
/// general `Reference`: only `type`/`uri`/`label`/`recorded_at`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(deny_unknown_fields)]
pub struct HandoffSignalSourceRef {
    #[serde(rename = "type")]
    pub ref_type: NonEmptyString,
    pub uri: NonEmptyString,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recorded_at: Option<IsoDateTime>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, RunxSchema)]
#[serde(deny_unknown_fields)]
#[runx_schema(id = "runx.handoff_signal.v1")]
pub struct HandoffSignal {
    pub schema: HandoffSignalSchema,
    pub signal_id: NonEmptyString,
    pub handoff_id: NonEmptyString,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boundary_kind: Option<NonEmptyString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_repo: Option<NonEmptyString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_locator: Option<NonEmptyString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_locator: Option<NonEmptyString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_locator: Option<NonEmptyString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outbox_entry_id: Option<NonEmptyString>,
    pub source: HandoffSignalSource,
    pub disposition: HandoffDisposition,
    pub recorded_at: IsoDateTime,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor: Option<HandoffSignalActor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<NonEmptyString>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_ref: Option<HandoffSignalSourceRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<crate::JsonObject>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
pub enum HandoffStateSchema {
    #[serde(rename = "runx.handoff_state.v1")]
    V1,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandoffStatus {
    AwaitingResponse,
    Engaged,
    NeedsRevision,
    Accepted,
    ApprovedToSend,
    Completed,
    Declined,
    Rerouted,
    Suppressed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(rename_all = "snake_case")]
pub enum SuppressionReason {
    RequestedNoContact,
    RemoveRequest,
    OperatorBlock,
    LegalRequest,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(deny_unknown_fields)]
#[runx_schema(id = "runx.handoff_state.v1")]
pub struct HandoffState {
    pub schema: HandoffStateSchema,
    pub handoff_id: NonEmptyString,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boundary_kind: Option<NonEmptyString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_repo: Option<NonEmptyString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_locator: Option<NonEmptyString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_locator: Option<NonEmptyString>,
    pub status: HandoffStatus,
    pub signal_count: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_signal_id: Option<NonEmptyString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_signal_at: Option<IsoDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_signal_disposition: Option<HandoffDisposition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suppression_record_id: Option<NonEmptyString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suppression_reason: Option<SuppressionReason>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}
