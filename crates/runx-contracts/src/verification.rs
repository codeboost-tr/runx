//! Verification contracts: checks and statuses for governed verification.
use serde::{Deserialize, Serialize};

use crate::Reference;
use crate::schema::{IsoDateTime, NonEmptyString, RunxSchema};

pub const VERIFICATION_SCHEMA: &str = "runx.verification.v1";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
pub enum VerificationSchema {
    #[serde(rename = "runx.verification.v1")]
    V1,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(rename_all = "snake_case")]
pub enum VerificationStatus {
    Passed,
    Failed,
    Pending,
    NotApplicable,
    Missing,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(deny_unknown_fields)]
pub struct VerificationCheck {
    pub check_id: NonEmptyString,
    pub criterion_ids: Vec<NonEmptyString>,
    pub status: VerificationStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<NonEmptyString>,
    #[serde(default)]
    pub checked_refs: Vec<Reference>,
    // Required on the wire (present, possibly empty); no serde default so Rust
    // deserialization matches the committed schema's `required` list.
    pub evidence_refs: Vec<Reference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verified_at: Option<IsoDateTime>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(deny_unknown_fields)]
#[runx_schema(id = "runx.verification.v1")]
pub struct Verification {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<VerificationSchema>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification_id: Option<NonEmptyString>,
    pub status: VerificationStatus,
    // Required on the wire (present, possibly empty); see `evidence_refs`.
    pub checks: Vec<VerificationCheck>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verified_at: Option<IsoDateTime>,
    // Required on the wire (present, possibly empty); no serde default so Rust
    // deserialization matches the committed schema's `required` list.
    pub evidence_refs: Vec<Reference>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(deny_unknown_fields)]
pub struct ReceiptVerificationSummary {
    pub signature_valid: bool,
    pub content_address_valid: bool,
    pub hash_commitments_valid: bool,
    pub authority_attenuation_valid: bool,
    pub criteria_bound: bool,
    pub redaction_valid: bool,
    pub external_attestations_present: bool,
}
