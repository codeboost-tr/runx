use runx_contracts::{AuthorityTerm, Decision, JsonObject};
use runx_core::policy::{PaymentRailAdmission, PaymentSpendCapabilityBinding, admit_payment_rail};
use runx_parser::GraphStep;

use super::inputs::{
    optional_bool_field, optional_typed_input, optional_typed_vec_input,
    require_non_empty_string_field, require_object_input, require_reference_input,
    required_typed_input,
};
use crate::RuntimeError;
use crate::adapter::SkillOutput;

pub(super) fn enforce_payment_receipt_before_success(
    step: &GraphStep,
    output: &SkillOutput,
    receipt: &runx_contracts::HarnessReceipt,
) -> Result<(), RuntimeError> {
    if !output.succeeded() || !payment_spend_step(step) {
        return Ok(());
    }
    let proof_present = receipt
        .harness
        .acts
        .iter()
        .any(|act| act.verification_refs.iter().any(is_payment_rail_proof_ref));
    if proof_present {
        return Ok(());
    }
    Err(RuntimeError::PaymentAuthorityDenied {
        step_id: step.id.clone(),
        reason: "payment:spend success requires a sealed rail proof reference".to_owned(),
    })
}

pub(super) fn payment_spend_step(step: &GraphStep) -> bool {
    step.scopes.iter().any(|scope| scope == "payment:spend")
}

pub(super) fn is_payment_rail_proof_ref(reference: &runx_contracts::Reference) -> bool {
    reference.reference_type == runx_contracts::ReferenceType::Verification
        && reference.label.as_deref() == Some("payment rail proof")
}

pub(super) fn enforce_payment_rail_admission_inputs(
    step: &GraphStep,
    inputs: &JsonObject,
) -> Result<(), RuntimeError> {
    let Some(input) = payment_rail_admission_inputs(step, inputs)? else {
        return Ok(());
    };
    let binding = input
        .spend_capability_binding
        .as_ref()
        .map(OwnedPaymentSpendCapabilityBinding::as_borrowed);
    let act_id = format!("act_{}", step.id);
    admit_payment_rail(PaymentRailAdmission {
        parent_authority: &input.parent_authority,
        child_authority: &input.child_authority,
        reservation_decision: input.reservation_decision.as_ref(),
        subset_proof_present: input.subset_proof_present,
        child_harness_ref: &input.child_harness_ref,
        act_id: &act_id,
        idempotency_key: Some(&input.idempotency_key),
        spend_capability_binding: binding,
        consumed_spend_capability_refs: &input.consumed_spend_capability_refs,
        spend_capability_ref: Some(&input.spend_capability_ref),
    })
    .map_err(|source| payment_authority_denied(step, source.to_string()))?;
    Ok(())
}

fn payment_rail_admission_inputs(
    step: &GraphStep,
    inputs: &JsonObject,
) -> Result<Option<OwnedPaymentRailAdmission>, RuntimeError> {
    if !payment_spend_step(step) {
        return Ok(None);
    }
    let reserved = require_object_input(step, inputs, "reserved_payment_authority")?;
    let idempotency = require_object_input(step, inputs, "idempotency")?;
    let reserved = parse_reserved_payment_authority(step, reserved)?;
    Ok(Some(OwnedPaymentRailAdmission {
        spend_capability_ref: require_reference_input(step, inputs, "spend_capability_ref")?,
        idempotency_key: require_non_empty_string_field(step, idempotency, "idempotency.key")?,
        parent_authority: reserved.parent_authority,
        child_authority: reserved.child_authority,
        reservation_decision: reserved.reservation_decision,
        subset_proof_present: reserved.subset_proof_present,
        child_harness_ref: reserved.child_harness_ref,
        spend_capability_binding: reserved.spend_capability_binding,
        consumed_spend_capability_refs: reserved.consumed_spend_capability_refs,
    }))
}

fn parse_reserved_payment_authority(
    step: &GraphStep,
    object: &JsonObject,
) -> Result<OwnedReservedPaymentAuthority, RuntimeError> {
    Ok(OwnedReservedPaymentAuthority {
        parent_authority: required_typed_input(
            step,
            object,
            "reserved_payment_authority.parent_authority",
            "parent_authority",
        )?,
        child_authority: required_typed_input(
            step,
            object,
            "reserved_payment_authority.child_authority",
            "child_authority",
        )?,
        reservation_decision: optional_typed_input(
            step,
            object,
            "reserved_payment_authority.reservation_decision",
            "reservation_decision",
        )?,
        subset_proof_present: optional_bool_field(step, object, "subset_proof_present")?
            .unwrap_or(false),
        child_harness_ref: required_typed_input(
            step,
            object,
            "reserved_payment_authority.child_harness_ref",
            "child_harness_ref",
        )?,
        spend_capability_binding: optional_typed_input(
            step,
            object,
            "reserved_payment_authority.spend_capability_binding",
            "spend_capability_binding",
        )?,
        consumed_spend_capability_refs: optional_typed_vec_input(
            step,
            object,
            "reserved_payment_authority.consumed_spend_capability_refs",
            "consumed_spend_capability_refs",
        )?
        .unwrap_or_default(),
    })
}

pub(super) fn payment_authority_denied(step: &GraphStep, reason: String) -> RuntimeError {
    RuntimeError::PaymentAuthorityDenied {
        step_id: step.id.clone(),
        reason,
    }
}

#[derive(Clone, Debug)]
pub(super) struct OwnedPaymentRailAdmission {
    parent_authority: AuthorityTerm,
    child_authority: AuthorityTerm,
    reservation_decision: Option<Decision>,
    subset_proof_present: bool,
    child_harness_ref: runx_contracts::Reference,
    spend_capability_binding: Option<OwnedPaymentSpendCapabilityBinding>,
    consumed_spend_capability_refs: Vec<runx_contracts::Reference>,
    spend_capability_ref: runx_contracts::Reference,
    idempotency_key: String,
}

#[derive(Clone, Debug)]
pub(super) struct OwnedReservedPaymentAuthority {
    parent_authority: AuthorityTerm,
    child_authority: AuthorityTerm,
    reservation_decision: Option<Decision>,
    subset_proof_present: bool,
    child_harness_ref: runx_contracts::Reference,
    spend_capability_binding: Option<OwnedPaymentSpendCapabilityBinding>,
    consumed_spend_capability_refs: Vec<runx_contracts::Reference>,
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct OwnedPaymentSpendCapabilityBinding {
    child_harness_ref: runx_contracts::Reference,
    act_id: String,
    reservation_decision_id: String,
    idempotency_key: String,
    amount_minor: u64,
    currency: String,
    counterparty: String,
    rail: String,
}

impl OwnedPaymentSpendCapabilityBinding {
    fn as_borrowed(&self) -> PaymentSpendCapabilityBinding<'_> {
        PaymentSpendCapabilityBinding {
            child_harness_ref: &self.child_harness_ref,
            act_id: &self.act_id,
            reservation_decision_id: &self.reservation_decision_id,
            idempotency_key: &self.idempotency_key,
            amount_minor: self.amount_minor,
            currency: &self.currency,
            counterparty: &self.counterparty,
            rail: &self.rail,
        }
    }
}
