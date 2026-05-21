---
spec_version: '2.0'
task_id: rust-approval-gate-naming-boundary
created: '2026-05-21T15:25:00Z'
updated: '2026-05-21T15:54:28Z'
status: completed
harden_status: passed
size: small
risk_level: medium
---

# Rust approval gate naming boundary

## Current State

Status: completed
Current phase: final
Next: done
Reason: the naming decision is closed: keep host protocol `ApprovalGate` and
authority-proof `approval_gate` as distinct public wire concepts, with no
schema label, field, or Rust type rename. Focused validation on
2026-05-21T13:54:28Z reconfirmed the cross-shape rejection tests and generated
schema drift check are green.
Blockers: none.
Allowed follow-up command: `none`
Latest runner update: 2026-05-21T15:54:28Z closed after focused validation
Review gate: pass

## Summary

Resolve naming for two distinct approval-related wire concepts:

- Host protocol approval gate:
  `ApprovalGate { id, reason, type?, summary? }`, carried by
  `ResolutionRequest.gate`.
- Authority-proof approval decision:
  `{ gate_id, gate_type, decision, reason? }`, carried under
  `authority_proof.approval_gate`.

The validation gate already asserts these shapes are not interchangeable. This
follow-up keeps their public Rust names and schema labels as-is with explicit
documentation. A rename would need a separate migration because both shapes are
already public wire contracts.

## Naming Decision

Keep both current public names and wire shapes:

- Host protocol `ApprovalGate` remains the request gate sent in
  `ResolutionRequest::Approval.gate`. It asks a host or caller to resolve an
  approval before execution continues.
- Authority-proof `approval_gate` remains the recorded approval decision under
  an `authority_proof`. It records the gate identifier, gate type, decision,
  and optional reason after policy evaluation.

No compatibility migration is needed because this is documentation-only. Do not
rename `ResolutionRequest.gate`, host `ApprovalGate`, or
`authority_proof.approval_gate` in this slice.

## Context

- `oss/crates/runx-contracts/src/host_protocol.rs` defines the host protocol
  `ApprovalGate`.
- `oss/packages/contracts/src/schemas/credentials.ts` defines the
  authority-proof approval decision shape.
- `rust-contract-schema-validation-gate` added negative schema tests so host
  gate shape is rejected inside `authority_proof.approval_gate`, and
  authority-proof approval-decision shape is rejected as a host
  `ResolutionRequest.gate`.
- Archived `rust-approval-gate-parity` completed local runtime approval parity;
  it did not own this cross-contract naming distinction.

## Objectives

- Decide whether both names remain or one concept gets a clearer public name.
- Preserve existing wire fields unless a harden-approved migration explicitly
  changes schemas and compatibility behavior.
- Document the distinction so implementers do not use the schema-validation
  gate as evidence that the concepts are the same.
- Keep host approval protocol validation and authority-proof schema validation
  separate.

## Scope

In scope:
- Naming and documentation for host approval gates versus authority-proof
  approval decisions.
- Optional Rust type alias, re-export, or doc change if harden chooses a
  no-wire-change clarification.
- Schema migration planning if a public schema name or field name must change.

Out of scope:
- Moving `AuthorityProof` types; owned by `rust-authority-proof-ownership`.
- Runtime approval behavior; archived `rust-approval-gate-parity` owns the
  completed local parity slice.
- Cloud approval routing or hosted HTTP routes.
- Changing authority-proof policy semantics.

## Dependencies

- `rust-contract-schema-validation-gate`
- `rust-approval-gate-parity` archive evidence
- `rust-authority-proof-ownership` for type-home decisions only

## Touchpoints

- `oss/crates/runx-contracts/src/host_protocol.rs`
- `oss/packages/contracts/src/schemas/credentials.ts`
- `oss/packages/contracts/src/schemas/resolution.ts`
- `oss/schemas/authority-proof.schema.json`
- `oss/schemas/resolution-request.schema.json`
- `oss/crates/runx-contracts/tests/schema_validation.rs`

## Risks

- Renaming a wire field would be a compatibility break unless handled through a
  deliberate schema migration.
- Keeping both names without documentation leaves the same confusion in place.
- Combining this with AuthorityProof ownership can overbroaden both changes.

## Acceptance

Profile: strict

Definition of done:
- [x] `dod1` The naming decision is explicit: keep both current public names
  with documentation, or rename one through a reviewed migration.
- [x] `dod2` Host `ApprovalGate` and authority-proof approval decision remain
  schema-distinct after the change.
- [x] `dod3` Any rename has a compatibility plan and validation evidence.
  No rename was performed, so no compatibility migration is needed.
- [x] `dod4` `rust-contract-schema-validation-gate` remains a validation gate,
  not the product-decision home for approval naming.

Validation:
- [x] `v1` Host and authority-proof schema confusion tests pass.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-contracts --test schema_validation -- --nocapture`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-21T13:54:28Z rerun passed 5 tests, including
    `host_approval_gate_is_rejected_inside_authority_proof` and
    `authority_proof_approval_gate_is_rejected_inside_host_resolution_request`.
- [x] `v2` Generated schemas stay current if schema labels change.
  - Command: `pnpm contracts:schemas:check`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-21T13:54:28Z rerun exited zero with no schema drift
    reported.

## Phase 1: Naming Decision

Goal: decide whether this is documentation-only or a rename.

Status: completed
Dependencies: none

Changes:
- Inventory public references to host `ApprovalGate` and authority-proof
  approval decision shapes.
- Decide whether the current names are acceptable with documentation.
- If renaming is required, define the exact migration and compatibility rules
  before touching source.

Acceptance:
- The selected naming path is written down and scoped.

## Phase 2: Clarification Or Migration

Goal: implement the selected naming path.

Status: completed
Dependencies: Phase 1

Changes:
- For documentation-only clarification, add the smallest docs/spec/type-doc
  updates needed to distinguish the concepts.
- For a rename, update schemas, Rust names, fixtures, and compatibility handling
  in one reviewed slice.
- Keep the negative schema tests that prove the two shapes are not
  interchangeable.

Acceptance:
- Implementers can tell which approval concept they are using from the type or
  documentation, and validation continues to reject cross-use.

## Rollback

If a rename creates compatibility risk or schema drift without a migration path,
drop the rename and keep both current wire names with explicit documentation.

## Origin

Created from `rust-contract-schema-validation-gate` Phase 3 after schema tests
proved the two approval concepts are distinct but left naming as a product
handoff.
