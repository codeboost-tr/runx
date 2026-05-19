---
spec_version: '2.0'
task_id: rust-approval-gate-parity
created: '2026-05-18T00:00:00Z'
updated: '2026-05-19T12:11:22Z'
status: draft
harden_status: not_run
size: large
risk_level: high
---

# Rust approval gate parity

## Current State

Status: draft
Current phase: none
Next: harden after scope reconciliation
Reason: draft created under `plans/rust-takeover.md`. Approval gates are the
governance surface customers notice; first-class spec rather than a runtime
sub-step.
Blockers: `rust-runtime-skeleton` complete, `rust-receipts-parity` complete,
and this draft must be narrowed to local runtime approval parity unless the
cloud approval HTTP contract source or a pinned artifact is present.
Allowed follow-up command: `scafld harden rust-approval-gate-parity`
Latest runner update: none
Review gate: not_started

## Summary

Land local approval-gate parity on the Rust runtime. The current TS
shape (`ApprovalGate { id, reason, type?, summary? }` plus
`ResolutionRequest { id, kind: "approval", gate }`,
`ResolutionResponse { actor: "human" | "agent", payload: unknown }`, and
`Caller.report` / `Caller.resolve`) is the cross-language contract. Rust must
not invent a parallel compatibility shape. `runx-runtime` consumes the
contract; sealed harness receipts capture the round-trip. Cloud approval
routing is not executable from this OSS checkout and is deferred to the
cloud HTTP contract stabilization path unless an exact pinned artifact is named.
Approval response payload is `unknown` at the published schema boundary;
approval requests must validate that payload to a boolean at the caller
boundary before resolving the gate.

This spec is the canary that proves Rust runtime can govern, not just
execute. It blocks any CLI cutover that touches mutation classes.

## Context

CWD: `.`

Packages:
- `@runxhq/core` (executor)
- `@runxhq/runtime-local` (runner-local approval, graph-governance)
- `@runxhq/contracts`
- `crates/runx-runtime`
- `crates/runx-contracts`

Current TypeScript sources:
- `packages/core/src/executor/index.ts` (ApprovalGate type)
- `packages/contracts/src/schemas/agent-act.ts` (ApprovalGate contract)
- `packages/contracts/src/schemas/resolution.ts` (approval resolution request
  and response contracts)
- `packages/runtime-local/src/runner-local/approval.ts`
- `packages/runtime-local/src/runner-local/graph-governance.ts`
- `packages/contracts/src/openapi-runtime.ts` (current OSS OpenAPI fragments for
  hosted approval objects)
- Cloud approval routes and durable-step code are external to this OSS checkout
  and read-only for this spec unless an exact pinned artifact is added.
- `crates/runx-contracts/src/host_protocol.rs` (existing Rust
  `ApprovalGate`, `ResolutionRequest`, `ResolutionResponse`)
- `crates/runx-contracts/src/redaction.rs`
- `crates/runx-receipts/src/verify/proof.rs`
- `crates/runx-receipts/src/canonical.rs`

Files impacted:
- `crates/runx-contracts/src/approval.rs` (new only if it re-exports or wraps
  the existing host-protocol types without creating a second wire model)
- `crates/runx-contracts/src/host_protocol.rs`
- `crates/runx-runtime/src/approval.rs`
- `fixtures/approval/**`
- `scripts/generate-rust-approval-fixtures.ts`

Invariants:
- The TS approval contract does not silently change. Any clarification that
  the Rust port forces (enumerated gate types, payload schema) lands in TS
  first via a small clarification spec, not by Rust drift.
- `ApprovalGate.type` and `ApprovalGate.summary` remain optional in Rust until
  TS makes them required. Rust serialization must omit absent optional fields,
  not emit nulls or empty objects to satisfy convenience tests.
- Approval resolution actors are exactly the TS enum values `human` and
  `agent`. Do not broaden to a free string in Rust.
- Approval resolution payload is boolean for approval requests. The Rust caller
  boundary must reject non-boolean payloads. Current TS runner-local paths are
  not uniform here, so this spec must include or depend on a small TS hardening
  task that replaces coercion/denial-by-default with explicit validation.
- Harness receipts capture every gate request, decision, actor, and gate hash.
- Approval receipt fixtures are proof-verifiable by
  `rust-receipt-proof-verification`; structural capture alone is not enough for
  this gate.
- Gate request, actor, boolean decision, idempotency key hash, and gate hash
  live in `runx-contracts` harness/decision/act fields and are included in the
  canonical receipt body digest commitment so post-hoc mutation is detectable.
  The implementation must rely on `runx-receipts` canonical body proof logic
  rather than introducing a receipt-local digest algorithm or a receipt-local
  approval envelope type.
- Approval summaries, receipt projection text, proof findings, and local error
  bodies must pass the existing secret/path redaction bar before persistence or
  display. Raw local absolute paths, bearer tokens, API keys, material refs,
  raw tokens, and raw secrets must not appear in fixtures, receipts, logs, or
  local diagnostics.
- Cloud HTTP routing remains out of scope for this local parity slice. A later
  cloud routing spec must document the HTTP contract before any Rust client
  consumes it.
- No approval bypasses: Rust runner must call the same gate evaluation paths
  as TS runner via shared `runx-core::policy` decisions.
- No legacy/compat readers: do not accept alternate field spellings, null
  optional fields, stringly boolean decisions, or old actor names unless TS
  accepts them in the named source contract first.

## Objectives

- Reuse the existing `runx-contracts::host_protocol` approval types (gate,
  request envelope, resolution envelope, actor identity) or re-export them from
  a single public home without creating a second wire model.
- Implement `runx-runtime` runner-side: gate emission, caller reporting,
  resolution awaiting, decision-to-receipt wiring.
- Add local cross-language fixtures: sandbox-escalation gate, graph-step scope
  gate, destructive-action gate, denied gate, expired gate.
- Update receipts to carry approval round-trip envelopes and proof-verifiable
  body commitments.
- Add at least one approval round-trip receipt fixture consumed by
  `rust-receipt-proof-verification`.
- Preserve the existing `runx-contracts::host_protocol` wire contract or move it
  through a single re-export path. Do not leave two public Rust approval models
  that can diverge.

## Scope

In scope:
- Contracts, runtime side, sealed harness receipt proof, and local caller
  approval round-trip fixtures.
- Local-only gate fixtures that prove request, report, resolve, denial,
  expiry, idempotency, and redaction behavior.

Out of scope:
- Aster operator UI consumption of gates (separate spec under aster v1 reset).
- Approval routing logic changes (the cloud rules stay in TS).
- Cloud approval POST/GET/PUT routes and Rust cloud client packaging until
  `cloud-http-contract-stabilization` provides a concrete source or pinned
  artifact.
- Replacing the TS runner-local approval path. Both runners co-exist until a
  TS sunset spec.

## Dependencies

- `rust-runtime-skeleton`, `rust-receipts-parity`, `rust-contracts-parity`.
- `rust-receipt-proof-verification` for approval receipt proof checks.
- `rust-ts-interop-boundary` for the cross-language crossing reference.
- `cloud-http-contract-stabilization` is a follow-up dependency for cloud
  approval routing, not a build blocker for this local runtime parity slice.

## Open Questions

- Cloud client packaging is explicitly deferred. This spec must not add
  `runx-cloud-client`, `crates/runx-runtime/src/cloud_client.rs`, or HTTP route
  assumptions.

## Build Readiness Hardening

Verdict: draft must be reconciled before harden/approval. The executable slice
is local runtime approval parity only; cloud approval routing remains blocked
until a concrete HTTP contract exists.

Blockers:
- **Cloud HTTP boundary is not buildable from this checkout.** This spec must
  not add or consume approval POST/GET/PUT routes, request bodies, response
  envelopes, auth headers, retry semantics, or cloud error envelopes. A later
  cloud approval routing spec may consume a pinned artifact with version and
  hash.
- **Contract shape is overspecified and risks Rust drift.** TS currently defines
  `ApprovalGate.type` and `summary` as optional
  (`packages/contracts/src/schemas/agent-act.ts`), and
  `ResolutionResponse.actor` as `human | agent`
  (`packages/contracts/src/schemas/resolution.ts`). The spec must require exact
  parity: no required gate type, no free-string actor, no null optional fields,
  and no secondary Rust approval wire shape.
- **Receipt proof integration is underspecified.** Metadata-only
  `approvalReceiptMetadata` in TS records `gate_id`, `gate_type`, `decision`,
  `reason`, and `summary`, but this spec requires proof-verifiable round-trip
  envelopes. Build must define where the approval request, actor, boolean
  decision, gate hash, idempotency key hash, redaction refs, and
  hash commitments live in the HarnessReceipt via `runx-contracts`
  harness/decision/act fields and must verify them through `runx-receipts`
  strict proof checks.
- **Idempotency is local-only in this slice.** Gate ids plus canonical gate hash
  must dedupe local caller requests. Cloud retry/idempotency semantics are a
  follow-up owned by the cloud routing contract.
- **Secret/path redaction is not yet part of the acceptance surface.** Approval
  summaries and local caller diagnostics can carry local paths or provider
  material. Acceptance must include negative fixtures proving raw
  `/Users/...`, Windows home paths, bearer tokens, API keys, material refs, raw
  tokens, and raw secrets are redacted before receipt persistence, proof-status
  projection, logs, and local error responses.
- **No-legacy rule needs to be explicit.** The build must reject compatibility
  shortcuts such as accepting `gate_type` in place of `type`, accepting
  `"true"`/`"false"` approval payload strings, accepting unknown actors, or
  silently ignoring extra fields on approval request/response envelopes. Add
  negative fixtures for `type: null`, `summary: null`, `gate_type`, unknown
  actor, string booleans, and extra fields because Rust `Option` fields can
  otherwise accidentally accept explicit JSON nulls.

Advisories:
- Prefer extending or re-exporting `crates/runx-contracts/src/host_protocol.rs`
  over adding `approval.rs` as an independent type home. A new module is fine
  only if `host_protocol` and public exports point at the same structs/enums.
- Do not introduce network dependencies in this slice. The runtime must have a
  local caller path that remains fully testable without network access.
- Fixture names should distinguish local approval pending, resolved-approved,
  resolved-denied, duplicate submit, duplicate resolve, expired approval,
  redacted summary, and tampered proof cases.

Required validation commands for the final build:

```sh
rg -n "export const approvalGateSchema|approvalResolutionRequestSchema|resolutionResponseActors" packages/contracts/src/schemas
rg -n "pub struct ApprovalGate|enum ResolutionRequest|struct ResolutionResponse|enum ResolutionResponseActor" crates/runx-contracts/src
! rg -n "approval.*POST|approval.*PUT|approval.*GET|runx-cloud-client|cloud_client" crates/runx-runtime crates/runx-contracts fixtures/approval
pnpm test -- --runInBand packages/contracts/src/index.test.ts packages/core/src/executor/index.test.ts packages/runtime-local/src/runner-local/process-sandbox.test.ts
cargo test -p runx-contracts approval -- --nocapture
cargo test -p runx-contracts host_protocol -- --nocapture
cargo test -p runx-runtime approval -- --nocapture
cargo test -p runx-runtime sandbox -- --nocapture
cargo test -p runx-receipts proof -- --nocapture
cargo test -p runx-receipts approval -- --nocapture
pnpm exec tsx scripts/generate-rust-approval-fixtures.ts --check
rg -n "type_null|summary_null|gate_type|unknown_actor|string_boolean|extra_field" fixtures/approval crates/runx-contracts crates/runx-runtime
pnpm boundary:check
pnpm rust:check
```

Additional proof/redaction validation required by this spec:

```sh
rg -n "approval.*gate_hash|idempotency_key_hash|redaction_refs|hash_commitments" fixtures/approval crates/runx-receipts crates/runx-runtime
! rg -n "/Users/|C:\\\\Users|bearer [A-Za-z0-9._:-]{6,}|sk-(proj-)?[A-Za-z0-9_-]{16,}|access[_-]?token|refresh[_-]?token|api[_-]?key|client[_-]?secret|material[_-]?ref|raw[_-]?(secret|token)" fixtures/approval crates/runx-runtime/src crates/runx-receipts/src
```
