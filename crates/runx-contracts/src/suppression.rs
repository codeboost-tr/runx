//! Suppression record contract (`runx.suppression_record.v1`): a do-not-contact
//! record scoped to a handoff, target, repo, or contact.
use serde::{Deserialize, Serialize};

use crate::handoff::SuppressionReason;
use crate::schema::{IsoDateTime, NonEmptyString, RunxSchema};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
pub enum SuppressionRecordSchema {
    #[serde(rename = "runx.suppression_record.v1")]
    V1,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(rename_all = "snake_case")]
pub enum SuppressionScope {
    Handoff,
    Target,
    Repo,
    Contact,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(deny_unknown_fields)]
#[runx_schema(id = "runx.suppression_record.v1")]
pub struct SuppressionRecord {
    pub schema: SuppressionRecordSchema,
    pub record_id: NonEmptyString,
    pub scope: SuppressionScope,
    pub key: NonEmptyString,
    pub reason: SuppressionReason,
    pub recorded_at: IsoDateTime,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<IsoDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_signal_id: Option<NonEmptyString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}
