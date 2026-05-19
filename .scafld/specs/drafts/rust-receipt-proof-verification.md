---
spec_version: '2.0'
task_id: rust-receipt-proof-verification
created: '2026-05-19T02:08:02Z'
updated: '2026-05-19T02:30:00Z'
status: draft
harden_status: not_run
size: medium
risk_level: high
---

# Rust receipt proof verification

## Current State

Status: draft
Current phase: none
Next: harden
Reason: receipt parity currently proves shape more than proof strength. The
next cut needs explicit digest, signature, redaction, external attestation, and
verification-summary checks before TypeScript receipts can be retired.
Blockers: `rust-receipts-parity` completed; contract-spine fixtures available.
Allowed follow-up command: `scafld harden rust-receipt-proof-verification`
Latest runner update: none
Review gate: not_started

## Summary

Turn `runx-receipts` from a structural receipt checker into a proof verifier.
The verifier must be able to recompute canonical receipt commitments, distinguish
body digests from full receipt digests, verify signatures through an explicit
verifier interface, and fail closed when receipt metadata claims proof strength
that the payload does not actually provide.

This spec closes the security gap that remains after basic Rust receipt parity:
a receipt that has the right fields is not necessarily trustworthy.

## Context

CWD: `.` (runx OSS workspace)

Relevant crates and fixtures:
- `crates/runx-receipts/src/canonical.rs`
- `crates/runx-receipts/src/error.rs`
- `crates/runx-receipts/src/signature.rs`
- `crates/runx-receipts/src/verify/**`
- `crates/runx-receipts/tests/**`
- `fixtures/contracts/harness-spine/**`

Known weak spots to close:
- Canonical JSON currently depends on generic serde serialization behavior.
- Receipt digests are not split cleanly between body commitments and full
  receipt archival fingerprints.
- `seal.digest`, `verification_summary`, signatures, redaction commitments, and
  external proof fields are not all treated as independently verifiable claims.
- Test fixtures do not yet include enough negative cases to catch proof drift.

Invariants:
- Verifier APIs fail closed. Missing verifier inputs are explicit errors, not
  soft warnings, whenever the receipt claims a signature or external proof.
- Public review output never leaks absolute local paths, secrets, env vars, or
  raw provider credentials.
- The proof layer remains generic runx core. Nitrosend routing, Slack text, and
  Sentry payload policy stay outside this crate.

## Objectives

- Define explicit canonical serialization for harness receipt proof material.
- Add separate body-digest and full-receipt-digest APIs.
- Recompute and verify `seal.digest` against the body commitment.
- Verify signatures through an injected verifier/key resolver, with deterministic
  test verifiers for fixtures.
- Verify `verification_summary` honesty: failed child checks, missing proofs, or
  malformed attestations cannot produce a successful summary.
- Add negative fixtures for digest tamper, signature tamper, redaction mismatch,
  missing external proof, and unsupported proof authority.

## Scope

In scope:
- Rust receipt proof APIs and tests.
- Contract-spine fixture expansion for proof-positive and proof-negative cases.
- Verifier finding codes for proof failures.
- Documentation of the proof model exposed to runtime, CLI, and Aster.

Out of scope:
- Persistent receipt store lookup or path discovery; owned by
  `rust-runtime-receipt-path-discovery`.
- Graph/tree traversal and child receipt lookup; owned by
  `rust-receipt-tree-resolution`.
- Cloud storage implementation.
- Nitrosend-specific Slack/GitHub comment formatting.

## Dependencies

- `runx-contract-spine-hard-cutover`.
- `rust-receipts-parity`.
- `rust-policy-authority-proof-parity` for final authority-proof semantics.
- Coordinates with `rust-receipt-tree-resolution`; either order can land, but
  final parent/child proof acceptance requires both.

## Assumptions

- Test keys and fixture-only verifiers are acceptable in deterministic fixtures,
  but production verification must use explicit verifier inputs.
- Existing archived receipts can remain archival artifacts; live governed paths
  must use post-cutover proof-verifiable receipts.

## Touchpoints

- `runx-receipts` canonical/digest modules.
- `runx-receipts` verification findings and summary output.
- Contract-spine fixture schema docs.
- Runtime and CLI callers that display receipt verification results.

## Risks

- Canonicalization drift could invalidate legitimate receipts if not versioned
  and fixture-backed.
- Treating missing proof material as a warning would create a false security
  signal.
- Signing the wrong material would allow metadata mutation without detection.

## Acceptance

Profile: strict

Validation:
- `cargo fmt --check --manifest-path crates/Cargo.toml`
- `cargo test --manifest-path crates/Cargo.toml -p runx-receipts`
- `cargo clippy --manifest-path crates/Cargo.toml -p runx-receipts --all-targets --all-features -- -D warnings`
- `git diff --check`

Required behavior:
- [ ] Canonical writer has fixture tests proving stable object ordering and
  deterministic output across repeated runs.
- [ ] Body digest excludes `signature`, `seal.digest`, and derived verification
  summaries; full digest includes immutable archival fields by explicit design.
- [ ] Receipt with a tampered body fails seal verification.
- [ ] Receipt with a tampered signature fails signature verification.
- [ ] Receipt with missing verifier inputs fails when signature verification is
  required.
- [ ] Receipt with a redaction commitment mismatch fails verification.
- [ ] Receipt that claims an external attestation without verifiable attestation
  material fails verification.
- [ ] Verification summary cannot claim success when any required proof check
  failed.
- [ ] Public verification output redacts local filesystem paths and raw secret
  values.

## Phase 1: Proof Model

Status: pending
Dependencies: none

Objective: Make the proof contract explicit before changing callers.

Changes:
- Define body/full digest semantics.
- Define required verifier inputs and failure modes.
- Document the exact receipt fields included in each proof commitment.

Acceptance:
- [ ] Proof contract documented in crate docs or receipt docs.
- [ ] Existing fixtures categorized as structural-only or proof-verifiable.

## Phase 2: Implementation

Status: pending
Dependencies: Phase 1

Objective: Implement proof checks in `runx-receipts`.

Changes:
- Add canonical writer and digest APIs.
- Add signature verifier trait or equivalent injected verifier boundary.
- Add proof finding codes and strict summary aggregation.

Acceptance:
- [ ] Positive proof fixture verifies.
- [ ] Negative proof fixtures fail with specific finding codes.

## Phase 3: Integration

Status: pending
Dependencies: Phase 2

Objective: Make runtime/CLI consumers display proof status without leaking
operator-local details.

Changes:
- Update receipt verification projections.
- Ensure CLI review text reports concise proof status and actionable failures.

Acceptance:
- [ ] CLI/runtime public output includes proof result, receipt id, and safe
  finding summaries.
- [ ] No absolute local paths appear in fixture-derived public output.

## Rollback

- Keep old structural verification behind tests until the proof verifier is
  green, then remove redundant code in the same change.
- If proof verification exposes fixture gaps, keep this spec open and add the
  missing fixtures instead of weakening the verifier.

## Review

Status: not_started
Verdict: none

Findings:
- none

## Self Eval

- Target score: 9.5. Passing means receipts are useful security evidence, not
  just structured logs.

## Deviations

- none

## Metadata

- created_by: scafld
- planning_reason: close receipt proof gaps before TS sunset and live Aster use

## Origin

Created by: scafld
Source: plan

## Harden Rounds

- none

## Planning Log

- 2026-05-19: Expanded placeholder into proof-verification contract after review
  of receipt parity and tree-resolution gaps.
