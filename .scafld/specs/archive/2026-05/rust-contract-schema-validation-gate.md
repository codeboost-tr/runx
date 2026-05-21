---
spec_version: '2.0'
task_id: rust-contract-schema-validation-gate
created: '2026-05-21T12:19:24Z'
updated: '2026-05-21T13:38:24Z'
status: completed
harden_status: not_run
size: medium
risk_level: high
---

# Rust contract schema validation gate

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-21T13:38:24Z
Review gate: pass

## Summary

Add a Rust-side schema validation gate that validates known fixture outputs
against generated JSON Schemas before or alongside serde roundtrip tests. Start
with authority-proof fixture output because it is currently produced by
`runx-core` policy code and only validated as generic JSON/object shape in
kernel fixture schemas.

This spec also records two contract-spine cleanup findings: host approval gates
and authority-proof approval decisions are distinct wire contracts with
confusingly similar names, and `AuthorityProof` output ownership lives outside
`runx-contracts`. This spec must not merge those shapes unless harden records a
product decision that they are the same concept.

## Context

Current schema ownership:
- `oss/packages/contracts/src/schemas/*.ts` owns TypeBox schemas.
- `oss/scripts/generate-contract-schemas.ts` emits `oss/schemas/*.json`.
- `oss/package.json` has `contracts:schemas:check`.

Current Rust validation:
- `oss/crates/runx-contracts/tests/*_fixtures.rs` parse fixture JSON into Rust
  structs and serialize back to the expected value.
- These tests prove examples roundtrip, but they do not validate every fixture
  against the generated schema.
- `oss/crates/runx-core` policy fixture tests compare generated authority-proof
  output to fixtures, while `oss/fixtures/kernel/schema/policy.schema.json`
  treats `expected.value` as generic JSON/object.

Observed drift risks:
- `oss/crates/runx-contracts/src/host_protocol.rs` defines
  `ApprovalGate { id, reason, type, summary }` for resolution requests.
- `oss/packages/contracts/src/schemas/credentials.ts` defines authority-proof
  approval gate as `{ gate_id, gate_type, decision, reason? }`.
- `oss/crates/runx-core/src/policy/types.rs` contains authority-proof policy
  wire types with mixed camelCase/snake_case surfaces rather than all authority
  proof types living in `runx-contracts`.

## Objectives

- Validate authority-proof fixture output against `oss/schemas/authority-proof.schema.json`
  as the first required schema gate.
- Expand the fixture-to-schema map to other contract fixtures where the payload
  schema is known and unambiguous.
- Keep serde roundtrip tests, but stop treating them as sufficient schema
  parity.
- Add an explicit fixture-to-schema mapping so each fixture's expected payload
  is checked by the schema that owns it.
- Record cleanup follow-ups or explicit product decisions for `ApprovalGate`
  shape naming and `AuthorityProof` type ownership.

## Scope

In scope:
- Add a Rust test or lightweight harness that loads generated schemas and
  validates existing contract fixture payloads.
- Add schema validation for `policy.buildAuthorityProofMetadata` fixture outputs
  against `authority-proof.schema.json`, validating the nested
  `expected.value.authority_proof` payload.
- Cover host-protocol, harness-spine, act-assignment, execution, aster-control,
  operational-policy, and other fixtures already consumed by `runx-contracts`
  only where fixture kind gives an unambiguous schema mapping.
- Add negative tests for known schema-invalid fixtures where available.
- Add negative tests proving host `ApprovalGate` shape is rejected inside
  `authority_proof.approval_gate`, and authority-proof approval-decision shape
  is rejected as a host `ResolutionRequest.gate`.
- Record cleanup blockers for gate naming and `AuthorityProof` ownership.

Out of scope:
- Inverting the schema pipeline to Rust `schemars` as source of truth.
- Rewriting all Rust contract types.
- Canonical JSON hashing; owned by `canonical-json-fingerprint-contract-v1`.
- TypeScript runtime-local sunset.

## Dependencies

- `canonical-json-fingerprint-contract-v1`
- `rust-contracts-parity`
- `rust-contract-spine-hard-cutover`
- `rust-ts-sunset-runtime-local`

## Touchpoints

- `oss/crates/runx-contracts/Cargo.toml`
- `oss/crates/runx-contracts/tests/`
- `oss/schemas/*.json`
- `oss/fixtures/contracts/**`
- `oss/packages/contracts/src/schemas/**`
- `oss/crates/runx-contracts/src/host_protocol.rs`
- `oss/crates/runx-core/src/policy/types.rs`

## Risks

- JSON Schema draft support must match generated schema features. Choosing a
  crate with partial draft 2020-12 support can create false confidence.
- Fixture wrappers have `fixture_kind` and `expected`; the schema gate must
  validate the correct nested value, not the wrapper unless the wrapper has its
  own schema.
- The cleanup findings are related but can overbroaden the validation gate if
  implemented in the same pass without harden approval.

## Acceptance

Profile: strict

Definition of done:
- [x] `dod1` Rust tests validate authority-proof fixture outputs against
  `oss/schemas/authority-proof.schema.json`.
- [x] `dod2` The schema validator and draft support are documented.
- [x] `dod3` Serde roundtrip tests still pass.
- [x] `dod4` Any fixture without a schema mapping is listed with an explicit
  reason.
- [x] `dod5` Host approval gate and authority-proof approval decision shapes are
  frozen as distinct surfaces or renamed through a follow-up spec.
- [x] `dod6` `AuthorityProof` ownership remains explicitly in `runx-core` or is
  promoted through a named follow-up spec.

Validation:
- [x] `v1` Generated schemas are current.
  - Command: `pnpm contracts:schemas:check`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-21 local command passed.
- [x] `v2` Rust contract schema validation tests pass.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-contracts schema -- --nocapture`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-21 local command passed the schema-filtered test run;
    `cargo test --manifest-path crates/Cargo.toml -p runx-contracts --test schema_validation -- --nocapture`
    passed all 5 schema-validation tests after Phase 2.
- [x] `v3` Existing Rust contract fixture tests still pass.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-contracts --tests -- --nocapture`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-21 local command passed.
- [x] `v4` TypeScript contract tests still pass.
  - Command: `pnpm vitest run packages/contracts/src`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-21 local command passed 64 tests.
- [x] `v5` Scafld validates this spec after Phase 3 handoff.
  - Command: `scafld validate rust-contract-schema-validation-gate --json`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-21 local command returned
    `{"ok":true,"command":"validate","result":{"task_id":"rust-contract-schema-validation-gate","valid":true,"errors":null}}`.
- [x] `v6` Contract fixture generator remains in sync with repaired fixtures.
  - Command: `pnpm fixtures:contracts:check`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-21 local command checked 4 act-assignment fixtures, 1
    Aster control fixture, 6 execution fixtures, and 26 host-protocol fixtures
    after updating `agentActResolutionRequest()`.

## Phase 1: Authority-Proof Schema Gate

Status: completed
Dependencies: none

Objective: Complete this phase.

Changes:
- Added `jsonschema = { version = "0.46.5", default-features = false }` as a dev-dependency for JSON Schema draft 2020-12 validation.
- Loaded `oss/schemas/authority-proof.schema.json` from tests.
- Validated policy fixture `expected.value.authority_proof` outputs.
- Added negative tests for host gate versus authority-proof approval-decision shape confusion.

Acceptance:
- none

## Phase 2: Coverage Closure

Status: completed
Dependencies: Phase 1

Objective: Complete this phase.

Changes:
- Added mappings for act-assignment, harness-spine, host-protocol resolution, and Aster control fixtures where each fixture kind has one generated schema or one explicit nested object/schema pair.
- Added an inventory guard over those fixture directories. Unmapped fixtures must use one of the declared exemption kinds: `event`, `execution_semantics`, `governed_act_ref`, `governed_disposition`, `input_context_capture`, `outcome_state`, `receipt_outcome`, `receipt_surface_ref`, `run_result`, or `run_state`.
- Fixed `fixtures/contracts/host-protocol/resolution-agent-act-request.json` to satisfy `agent-act-invocation.schema.json` by carrying the full `agent_context_envelope` payload required by the generated schema.
- Updated `scripts/generate-rust-contract-fixtures.ts` so
  `agentActResolutionRequest()` generates that full envelope and
  `fixtures:contracts:check` cannot regress it.

Acceptance:
- none

## Phase 3: Contract Cleanup Handoff

Status: completed
Dependencies: Phase 2

Objective: Complete this phase.

Changes:
- Deferred approval-gate naming to `rust-approval-gate-naming-boundary`.
- Deferred `AuthorityProof` Rust type ownership to `rust-authority-proof-ownership`.
- Recorded that this validation gate does not merge host approval gates with authority-proof approval decisions and does not move `AuthorityProof` wire types out of `runx-core`.

Acceptance:
- none

## Rollback

If the validator crate cannot faithfully handle the generated schema draft,
stop and record the blocker. Do not complete with a validator that silently
ignores schema features used by `oss/schemas`.

## Review

Status: completed
Verdict: pass
Mode: verify
Provider: claude:claude-opus-4-7
Output: claude.mcp_submit_review
Summary: Prior high blocker `schema-gate-fixture-generator-drift` is fully repaired. `agentActResolutionRequest()` in `scripts/generate-rust-contract-fixtures.ts:808-831` now emits the full `agent_context_envelope` payload (allowed_tools, current_context, historical_context, inputs, instructions, provenance, run_id, skill, step_id, trust_boundary). After running `stableJson()` (alpha-sorted key serialization), the generated bytes match `fixtures/contracts/host-protocol/resolution-agent-act-request.json` exactly, so `fixtures:contracts:check` and the new schema gate agree. The two negative tests remain semantically correct: the host-gate shape (id/reason/type/summary) is rejected by `authority-proof.schema.json`'s `approval_gate` (requires gate_id/gate_type/decision, additionalProperties:false), and the authority-proof gate shape is rejected by all three `anyOf` branches of `resolution-request.schema.json`. All schema files referenced by the new mappings exist. No regressions in scope.

Attack log:
- `scripts/generate-rust-contract-fixtures.ts:808`: Verify the prior blocker repair: confirm agentActResolutionRequest() now emits the full agent_context_envelope and stableJson output matches resolution-agent-act-request.json byte-for-byte -> clean (Generator at 808-831 emits allowed_tools, current_context, historical_context, inputs, instructions, provenance, run_id, skill, step_id, trust_boundary. stableJson sorts keys alphabetically; the traced output equals the single-line fixture content exactly. fixtures:contracts:check (byte-for-byte) will pass.)
- `crates/runx-contracts/tests/schema_validation.rs:233-277`: Confirm negative tests still semantically reject opposing gate shapes after repair -> clean (Host-shape injection fails authority-proof approval_gate (required gate_id/gate_type/decision + additionalProperties:false). Authority-proof shape fails all three anyOf branches of resolution-request.schema.json: input branch lacks questions, approval branch's gate requires id+reason and forbids extras, agent_act branch lacks invocation.)
- `crates/runx-contracts/tests/schema_validation.rs:29-153`: Validate schema mapping completeness: every referenced schema file exists and every unmapped fixture in surveyed directories has an exempt fixture_kind -> clean (All 16 schema files referenced by CONTRACT_FIXTURE_SCHEMA_MAPPINGS exist under schemas/. Spot-checked unmapped fixtures (event-*, inspect-host-state-*, result-host-run-*, governed-disposition, outcome-state, etc.) all carry a fixture_kind that is in CONTRACT_FIXTURE_EXEMPT_KINDS.)
- `crates/runx-contracts/Cargo.toml:27`: Confirm jsonschema dev-dependency declaration uses draft 2020-12 capable version with default-features disabled -> clean (jsonschema 0.46.5 declared with default-features=false; tests use jsonschema::draft202012::options().build(...). Schemas declare $schema draft/2020-12; no external $ref in resolution-request.schema.json or authority-proof.schema.json so default resolver suffices.)
- `.scafld/specs/active/rust-contract-schema-validation-gate.md`: Scope drift: confirm task changes since approval baseline remain inside declared touchpoints and ambient drift is not attributed to this task -> clean (Task-scoped changes (schema_validation.rs, resolution-agent-act-request.json, generate-rust-contract-fixtures.ts) all sit inside declared touchpoints. Ambient drift (payments, sandbox, x402, mcp, skill-author-runtime) is unrelated and not in scope.)

Findings:
- none

## Origin

User-provided cross-scan synthesis on 2026-05-21 identified TypeBox schemas as
the current oracle and Rust contracts as a hand-written copy without schema
validation.
