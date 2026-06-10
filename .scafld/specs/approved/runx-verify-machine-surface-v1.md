---
spec_version: '2.0'
task_id: runx-verify-machine-surface-v1
created: '2026-06-10T05:08:42Z'
updated: '2026-06-10T05:16:21Z'
status: approved
harden_status: not_run
size: medium
risk_level: medium
---

# Machine-grade single-receipt verify surface

## Current State

Status: approved
Current phase: none
Next: build
Reason: draft created
Blockers: none
Allowed follow-up command: `scafld build runx-verify-machine-surface-v1`
Latest runner update: none
Review gate: not_started

## Summary

Make `runx verify` consumable by machines, one receipt at a time. The hosted
receipt notary (spec `hosted-receipt-notary-v1` in the private root) verifies
edge-sealed receipts by invoking the runx binary, never by reimplementing
verification in TypeScript. That requires `runx verify` to accept a single
receipt from a file or stdin, emit a stable machine-readable JSON verdict, and
ship a conformance fixture corpus that any embedding surface can replay to
prove its verifier matches the CLI byte-for-byte.

This is the phase-1 "binary is the source of truth" guarantee: there is one
compiled verifier, and every surface that claims to verify a runx receipt
calls it.

## Objectives

- `runx verify --receipt <path>` and `runx verify --receipt -` (stdin) verify
  exactly one receipt document without requiring a receipt store directory.
- `--json` emits a stable verdict object: schema name, receipt id, digest
  validity, content-address validity, signature mode and outcome, findings
  (code/path/message), and a single top-level `valid` boolean.
- Exit codes are contractual: 0 valid, 1 invalid, 64 usage error.
- The verdict JSON shape gets a named schema id (e.g.
  `runx.verify_verdict.v1`); if runx-contracts schema emission is the
  established pattern for machine shapes, emit it there, otherwise lock the
  shape with fixture tests in the CLI crate.
- A conformance corpus of fixture receipts (valid, tampered body, tampered
  signature, unknown key, broken lineage, malformed JSON) lives in
  `oss/fixtures/receipt-verify/` with the expected verdict for each, and
  tests replay the corpus through both the CLI surface and the library API.
- Store-mode tree verification semantics are unchanged.

## Scope

In scope:

- `crates/runx-cli/src/verify.rs` single-receipt input, stdin support, verdict
  JSON output.
- `crates/runx-cli/src/launcher.rs` help text for the new flag.
- Conformance fixtures under `oss/fixtures/receipt-verify/` plus replay tests
  in runx-cli and runx-receipts.
- Docs note in `docs/security-authority-proof.md` (Offline Receipt
  Verification section).

Out of scope:

- Any hosted/notary code (separate private-root spec).
- Changes to receipt wire schemas or sealing.
- Changes to store-mode tree verification behavior.
- Networked verification or key distribution.

## Dependencies

- Offline verify command on main:
  [crates/runx-cli/src/verify.rs](/Users/kam/dev/runx/runx/oss/crates/runx-cli/src/verify.rs)
- Pure verification in runx-receipts:
  [crates/runx-receipts/src/verify.rs](/Users/kam/dev/runx/runx/oss/crates/runx-receipts/src/verify.rs)
- Existing safe-projection precedent in
  [crates/runx-receipts/tests/receipt_contracts.rs](/Users/kam/dev/runx/runx/oss/crates/runx-receipts/tests/receipt_contracts.rs)

## Grounding Evidence

- `runx verify` exists with store-dir mode, lineage-tree grouping, production
  signature verification via `RUNX_RECEIPT_VERIFY_KID` /
  `RUNX_RECEIPT_VERIFY_ED25519_PUBLIC_KEY_BASE64`, and non-zero exit on
  findings.
  [crates/runx-cli/src/verify.rs](/Users/kam/dev/runx/runx/oss/crates/runx-cli/src/verify.rs)
- Verification primitives are pure and complete in runx-receipts:
  `verify_receipt`, `verify_receipt_proof`, `receipt_id_is_content_addressed`,
  and the `ReceiptFindingCode` vocabulary.
  [crates/runx-receipts/src/verify/finding.rs](/Users/kam/dev/runx/runx/oss/crates/runx-receipts/src/verify/finding.rs:10)
- Receipt issuer types already distinguish Local and Hosted issuers, which the
  notary counter-seal relies on downstream.
  [crates/runx-contracts/src/receipt.rs](/Users/kam/dev/runx/runx/oss/crates/runx-contracts/src/receipt.rs:282)
- Fixture receipts with valid/abnormal shapes already exist under
  [crates/runx-receipts/fixtures/contracts/harness-spine](/Users/kam/dev/runx/runx/oss/crates/runx-receipts/fixtures/contracts/harness-spine)
  and can seed the conformance corpus.

## Assumptions

- The executor may be Codex; record evidence through
  `scafld build runx-verify-machine-surface-v1` after each phase and run
  `scafld review` with a real provider before `scafld complete`.
- Use `CARGO_TARGET_DIR=target/runx-verify-machine-surface` for all cargo
  commands to avoid contending with other agents' builds.
- Single-receipt mode verifies digest, content address, structure, and
  signature. Lineage findings that require sibling receipts are reported as
  an explicit `lineage_unverified` informational state, not failures, because
  a single document cannot prove tree membership. Store mode remains the tree
  authority.
- stdin/file input is size-capped (reject above ~10 MiB) with a usage-class
  error so the surface cannot be memory-bombed.
- The verdict shape is additive-stable: fields may be added, never repurposed;
  semantic changes require a new schema id version.

## Touchpoints

- [crates/runx-cli/src/verify.rs](/Users/kam/dev/runx/runx/oss/crates/runx-cli/src/verify.rs)
- [crates/runx-cli/src/launcher.rs](/Users/kam/dev/runx/runx/oss/crates/runx-cli/src/launcher.rs)
- [crates/runx-receipts/src/verify.rs](/Users/kam/dev/runx/runx/oss/crates/runx-receipts/src/verify.rs)
- [fixtures/](/Users/kam/dev/runx/runx/oss/fixtures) (new `receipt-verify/` corpus)
- [docs/security-authority-proof.md](/Users/kam/dev/runx/runx/oss/docs/security-authority-proof.md)

## Risks

- Verdict shape churn would break the notary contract downstream. Mitigation:
  fixture-locked verdict tests and a named schema id from day one.
- Single-receipt mode could silently weaken tree semantics. Mitigation:
  explicit `lineage_unverified` reporting; store mode untouched.
- Parsing attacker-supplied receipts from stdin is an attack surface.
  Mitigation: size cap, existing serde strictness (`deny_unknown_fields` on
  receipt types), malformed-input fixtures in the corpus.

## Acceptance

Profile: strict

Validation:
- `cd crates && cargo fmt --check`
- `cd crates && CARGO_TARGET_DIR=target/runx-verify-machine-surface cargo test -p runx-cli verify`
- `cd crates && CARGO_TARGET_DIR=target/runx-verify-machine-surface cargo test -p runx-cli launcher`
- `cd crates && CARGO_TARGET_DIR=target/runx-verify-machine-surface cargo test -p runx-receipts`
- `git diff --check`

## Phase 1: Single-Receipt Input And Verdict JSON

Status: pending
Dependencies: none

Objective: One receipt in, one stable verdict out.

Changes:
- Add `--receipt <path|->` to `runx verify`; mutually exclusive with a
  positional store receipt id.
- Implement the verdict JSON object with a named schema id; `--json` in
  single-receipt mode emits exactly one verdict document.
- Exit codes: 0 valid, 1 invalid, 64 usage.
- Enforce the input size cap with a usage-class error.
- Unit tests for valid, tampered, oversized, and malformed inputs.

Acceptance:
- [ ] `ac1` command - CLI verify tests pass with single-receipt mode
  - Command: `cd crates && CARGO_TARGET_DIR=target/runx-verify-machine-surface cargo test -p runx-cli verify`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `ac2` command - Launcher help/routing tests pass
  - Command: `cd crates && CARGO_TARGET_DIR=target/runx-verify-machine-surface cargo test -p runx-cli launcher`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Phase 2: Conformance Corpus

Status: pending
Dependencies: phase1

Objective: Any embedding surface can prove it verifies exactly like the CLI.

Changes:
- Build `fixtures/receipt-verify/` with at least: valid production-signed,
  tampered body, tampered signature, unknown kid, broken lineage reference,
  malformed JSON, plus an expected-verdict JSON per fixture.
- Add a replay test in runx-cli running every corpus entry through the
  single-receipt surface, asserting the exact expected verdict.
- Add a runx-receipts test replaying the same corpus through the library API
  so the CLI and library can never drift.
- Document the corpus as the notary's conformance gate in
  `docs/security-authority-proof.md`.

Acceptance:
- [ ] `ac3` command - Corpus replay passes through the CLI surface
  - Command: `cd crates && CARGO_TARGET_DIR=target/runx-verify-machine-surface cargo test -p runx-cli verify`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `ac4` command - Corpus replay passes through the library API
  - Command: `cd crates && CARGO_TARGET_DIR=target/runx-verify-machine-surface cargo test -p runx-receipts`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Phase 3: Final Gate

Status: pending
Dependencies: phase2

Objective: Formatting and whitespace clean; no store-mode regression.

Changes:
- Run formatting and the focused test list.
- Confirm store-mode tree verification output is unchanged under existing
  tests.

Acceptance:
- [ ] `ac5` command - Rust formatting is clean
  - Command: `cd crates && cargo fmt --check`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `ac6` command - Diff has no whitespace errors
  - Command: `git diff --check`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Follow-Up Specs

- `hosted-receipt-notary-v1` (private root): consumes this surface as the
  notary's verifier; its build is blocked until this spec completes.

## Rollback

- Revert the new flag and corpus together; store-mode behavior is untouched so
  rollback cannot regress existing verification.
- A retired verdict schema id must never be reused with different semantics;
  bump the version instead.

## Review

Status: not_started
Verdict: none

Findings:
- none

## Self Eval

- none

## Deviations

- none

## Metadata

- created_by: scafld

## Origin

Created by: scafld
Source: plan

## Harden Rounds

- none

## Planning Log

- 2026-06-10: Authored as the dependency root of the phase-1 connector-hosting
  plan: the hosted notary verifies via the compiled runx binary, so the binary
  needs a machine-grade single-receipt surface and a conformance corpus first.
