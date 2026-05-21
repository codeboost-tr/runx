---
spec_version: '2.0'
task_id: rust-authority-proof-ownership
created: '2026-05-21T15:25:00Z'
updated: '2026-05-21T23:54:56+10:00'
status: completed
harden_status: not_run
size: medium
risk_level: high
---

# Rust AuthorityProof ownership

## Current State

Status: completed
Current phase: final
Next: done
Reason: AuthorityProof remains explicitly policy-owned in `runx-core`.
`runx-contracts` validates the emitted payload against the generated schema,
but does not own the Rust wire structs because the proof is produced only by
policy and shares policy admission support types. Focused validation on
2026-05-21T23:54:56+10:00 reconfirmed schema validation, policy fixture
parity, and generated-schema drift checks are green.
Blockers: none for this ownership decision. Do not broaden this into
contract-pipeline inversion.
Allowed follow-up command: `none`
Latest runner update: 2026-05-21T23:54:56+10:00 closed after focused validation
Review gate: pass

## Summary

Decide the long-term Rust home for `AuthorityProof` wire types. Today the
schema is owned by the TypeBox contracts package and emitted as
`oss/schemas/authority-proof.schema.json`, while the Rust authority-proof output
types live in `oss/crates/runx-core/src/policy/types.rs` because they were
ported with policy behavior. That split is acceptable for the schema validation
gate, but it is not an ownership decision.

This spec keeps `AuthorityProof` explicitly policy-owned in `runx-core`.
Promotion into `runx-contracts` is deferred because the current wire structs are
emitted by policy-only behavior and share policy admission types. The generated
schema remains the contract guard for the emitted JSON shape.

## Context

- `rust-contract-schema-validation-gate` validates
  `policy.buildAuthorityProofMetadata` fixture outputs against
  `oss/schemas/authority-proof.schema.json`.
- `oss/crates/runx-core/src/policy/types.rs` currently contains the Rust
  authority-proof policy wire types.
- `oss/crates/runx-contracts` owns most other cross-crate contract structs.
- `rust-contract-pipeline-inversion` may later make Rust contract types the
  schema source of truth, and it needs a settled `AuthorityProof` home before
  inversion can cover this schema cleanly.

## Objectives

- Decide whether `AuthorityProof` remains explicitly in `runx-core` or moves to
  `runx-contracts`.
- If it remains in `runx-core`, document why policy ownership is intentional and
  add a guard so future contract-spine work does not assume the type was
  forgotten.
- If it moves to `runx-contracts`, move only the wire types and re-export or
  consume them from policy without changing JSON field names, optionality, enum
  values, or fixture output.
- Keep `authority-proof.schema.json` validation passing throughout the decision.

## Scope

In scope:
- Rust ownership and module boundary for authority-proof wire types.
- Re-export strategy if the types move.
- Schema and fixture validation proving no wire-shape change.
- Documentation of the chosen boundary for later contract-pipeline inversion.

Out of scope:
- Approval-gate naming between host approval gates and authority-proof approval
  decisions; owned by `rust-approval-gate-naming-boundary`.
- Changing authority-proof semantics, credential binding behavior, sandbox
  summaries, or public-work policy.
- Inverting the full TypeBox-to-Rust schema pipeline; owned by
  `rust-contract-pipeline-inversion`.
- Runtime-local TypeScript sunset work.

## Dependencies

- `rust-contract-schema-validation-gate`
- `rust-contract-pipeline-inversion`
- `rust-policy-authority-proof-parity` archive evidence

## Touchpoints

- `oss/crates/runx-core/src/policy/types.rs`
- `oss/crates/runx-core/src/policy/authority_proof.rs`
- `oss/crates/runx-contracts/src/`
- `oss/crates/runx-contracts/tests/schema_validation.rs`
- `oss/fixtures/kernel/policy/*authority-proof*.json`
- `oss/schemas/authority-proof.schema.json`

## Risks

- Moving types can accidentally alter serde casing or omitted-field behavior.
  Treat schema validation and fixture parity as required gates.
- Keeping the types in `runx-core` without an explicit architectural note leaves
  future contract work to rediscover the exception.
- Sharing helper enums too aggressively can couple policy-only concepts to
  contract crates before the public contract surface requires it.

## Decision

`AuthorityProof` remains `runx-core` policy-owned.

Rationale:
- The only Rust producer is `policy.buildAuthorityProofMetadata` in
  `runx-core`.
- The output shares policy-owned support types: `ScopeAdmission`,
  `AuthorityKind`, and `CredentialGrantReference`.
- `runx-contracts` already carries the executable contract guard by validating
  policy fixture outputs against `schemas/authority-proof.schema.json`.
- Moving only the top-level structs would either split tightly coupled policy
  admission types or expand this slice into a broader policy/contract migration.

Guardrail:
- `crates/runx-core/src/policy/types.rs` documents the policy-owned exception at
  the `AuthorityProof` wire structs.
- `docs/security-authority-proof.md` records the ownership boundary for future
  contract-spine and schema-pipeline inversion work.
- `crates/runx-contracts/tests/schema_validation.rs` remains the schema guard
  for `runx.authority-proof.v1` fixture output.

## Acceptance

Profile: strict

Definition of done:
- [x] `dod1` The chosen `AuthorityProof` Rust owner is explicit:
  `runx-core` policy-owned or `runx-contracts` contract-owned.
- [x] `dod2` If types move, authority-proof fixtures and schema validation pass
  with no JSON diff. Not applicable to this implementation because the types
  stayed in `runx-core`; schema validation and fixture parity still passed.
- [x] `dod3` If types stay, docs or spec evidence states why this schema remains
  a policy-owned exception.
- [x] `dod4` `rust-contract-pipeline-inversion` can point at this decision
  instead of carrying a conditional `AuthorityProof` caveat.

Validation:
- [x] `v1` Authority-proof schema validation still passes.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-contracts --test schema_validation -- --nocapture`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-21T23:54:56+10:00 rerun passed 5 tests, including
    `authority_proof_outputs_validate_against_generated_schema`.
- [x] `v2` Policy fixture parity still passes.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-core policy_fixtures -- --nocapture`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-21T23:54:56+10:00 rerun passed
    `policy_fixtures_match_rust_policy`.
- [x] `v3` Generated schemas stay current.
  - Command: `pnpm contracts:schemas:check`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-21T23:54:56+10:00 rerun exited zero with no schema drift
    reported.

## Phase 1: Ownership Decision

Goal: pick the type home without moving code.

Status: completed
Dependencies: none

Changes:
- Inventoried Rust and schema consumers of authority-proof wire types.
- Chose explicit `runx-core` policy ownership.
- Recorded why the selected boundary is stable enough for later schema-pipeline
  inversion: the contract-spine can point to this exception until a full
  policy/contract migration can preserve the JSON shape.

Acceptance:
- The decision is written down before any type move occurs.

## Phase 2: Boundary Implementation

Goal: implement only the chosen boundary.

Status: completed
Dependencies: Phase 1

Changes:
- Kept `runx-core` ownership and added documentation/guardrails only.
- Did not move wire structs/enums; serde output is unchanged.
- Ran schema, fixture, and generated-schema validation after the boundary
  change.

Acceptance:
- The chosen owner is enforceable and authority-proof JSON stays unchanged.

## Rollback

If a promotion changes generated JSON, fixture output, or public imports beyond
the chosen boundary, revert the promotion and keep `runx-core` ownership
explicit until a smaller move can preserve the wire shape.

## Origin

Created from `rust-contract-schema-validation-gate` Phase 3. That validation
gate proved schema compatibility but intentionally did not move authority-proof
types or decide product ownership.
