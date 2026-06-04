//! Cross-language parity for `compute_maturity`.
//!
//! The Rust maturity decision reads the shared fixture that previously guarded
//! the TypeScript mirror. Keeping the fixture preserves the public case table
//! while `runx_core::policy::compute_maturity` remains the sole implementation.

use runx_contracts::maturity::{MaturitySignals, MaturityTier};
use runx_core::policy::compute_maturity;
use serde::Deserialize;

#[derive(Deserialize)]
struct ParityCase {
    name: String,
    signals: MaturitySignals,
    expected: MaturityTier,
}

const CASES_JSON: &str =
    include_str!("../../../fixtures/kernel/maturity/compute-maturity-cases.json");

#[test]
fn compute_maturity_matches_cross_language_fixture() -> Result<(), Box<dyn std::error::Error>> {
    let cases: Vec<ParityCase> = serde_json::from_str(CASES_JSON)?;
    assert!(
        !cases.is_empty(),
        "compute-maturity-cases.json must declare at least one case"
    );
    for case in cases {
        assert_eq!(
            compute_maturity(&case.signals),
            case.expected,
            "case {}: Rust compute_maturity diverged from the shared parity fixture",
            case.name
        );
    }
    Ok(())
}
