use std::env;
use std::path::PathBuf;

use runx_contracts::JsonValue;
use runx_pay::{
    DeterministicPaymentFinalitySupervisor, PaymentRuntimeEffect,
    ledger::{X402_PAY_PAYMENT_PROFILE, persist_x402_payment_ledger_projection_event},
};
use runx_runtime::{
    HarnessReplayOutput, LocalOrchestrator, ProviderPermissionEffect, RUNX_RECEIPT_DIR_ENV,
    RuntimeEffectRegistry,
};

#[must_use]
pub fn local_orchestrator() -> LocalOrchestrator {
    LocalOrchestrator::with_effects(payment_effect_registry())
}

#[must_use]
pub fn payment_effect_registry() -> RuntimeEffectRegistry {
    let mut registry = RuntimeEffectRegistry::with_effect(PaymentRuntimeEffect::new(
        DeterministicPaymentFinalitySupervisor,
    ));
    let _ = registry.register_effect(ProviderPermissionEffect);
    registry
}

pub fn persist_payment_ledger_projection(output: &HarnessReplayOutput) -> Result<(), String> {
    if metadata_string(output, "payment_ledger_profile") != Some(X402_PAY_PAYMENT_PROFILE) {
        return Ok(());
    }
    let Some(receipt_dir) = env::var_os(RUNX_RECEIPT_DIR_ENV).map(PathBuf::from) else {
        return Ok(());
    };
    let scenario_id = metadata_string(output, "payment_ledger_scenario_id")
        .ok_or_else(|| "metadata.payment_ledger_scenario_id is required".to_owned())?;
    persist_x402_payment_ledger_projection_event(
        receipt_dir,
        &format!("gx_{}", output.fixture.name),
        output.receipt.created_at.as_str(),
        &output.receipt,
        &output.steps,
        scenario_id,
    )
    .map(|_| ())
    .map_err(|error| error.to_string())
}

fn metadata_string<'a>(output: &'a HarnessReplayOutput, key: &str) -> Option<&'a str> {
    output
        .fixture
        .metadata
        .get(key)
        .and_then(|value| match value {
            JsonValue::String(value) => Some(value.as_str()),
            _ => None,
        })
}
