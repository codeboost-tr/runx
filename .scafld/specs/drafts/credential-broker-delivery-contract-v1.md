---
spec_version: '2.0'
task_id: credential-broker-delivery-contract-v1
created: '2026-05-22T00:28:36+10:00'
updated: '2026-05-22T00:52:00+10:00'
status: draft
harden_status: not_run
size: large
risk_level: high
---

# Credential broker and delivery contract v1

## Current State

Status: draft
Current phase: Phase 1 contract shape and Phase 2a built-in runtime adoption
landed
Next: external adapter/outbox consumption after broker response metadata can be
threaded into supervised invocations
Reason: Rust has a runtime `CredentialDelivery` secret channel for built-in
`cli-tool` and MCP adapter process spawn, but the shared broker/delivery wire
contract is not ratified for external execution adapters, outbox/provider
adapters, or future hosted delivery. The external execution-adapter protocol
must consume a credential-delivery primitive; it must not become the credential
broker itself.
Blockers: Phase 3 external adapter/outbox adoption remains blocked because
`SkillInvocation` carries private `CredentialDelivery` only, not the public
credential-delivery broker response/observation metadata needed to publish
receipt-safe credential refs. Do not serialize raw credential material into
external adapter invocations as a shortcut.
Allowed follow-up command: `scafld harden credential-broker-delivery-contract-v1`
Latest runner update: 2026-05-22T00:52:00+10:00 added runtime adoption for the
safe built-in slice: runtime profiles map from the new contract profile, empty
secret material fails closed before env injection, cli-tool redacts before
final truncation, and MCP real process transport proves process-env delivery
and output redaction.
Review gate: phase1_contract_and_phase2a_runtime_ready

## Summary

Define the shared credential broker and delivery contract used by Rust-supervised
execution lanes. The contract starts after policy admission and authority proof:
Rust decides whether a grant may bind to a run, resolves the opaque
`material_ref` through a trusted broker/resolver, maps the material to a
declared delivery profile, and gives the child or provider adapter only the
scoped secret material it is allowed to receive.

This is not a general auth plugin protocol and not a secret store. It is the
runtime handoff primitive between admitted authority and a supervised side-effect
boundary. Raw secret material must never appear in manifests, invocation frames,
authority proofs, receipts, logs, captured output, public projections, or
adapter observations.

The completed `rust-adapter-credential-delivery` spec proved the first local
mechanics: `CredentialDelivery`, `MaterialResolver`, provider env profiles,
`SecretEnv`, and redaction for built-in `cli-tool` and MCP paths. This spec
turns that implementation idea into a stable contract that can be reused by:

- skill author subprocess execution through the existing `CredentialDelivery`
  channel;
- external execution adapters under `external-adapter-plugin-protocol-v1`;
- thread/outbox provider adapters that need provider tokens for comments, PR
  updates, or publication;
- hosted/cloud runtimes that broker secrets outside the local process.

## Context

Existing implemented Rust pieces:
- `crates/runx-runtime/src/credentials.rs` defines `CredentialDeliveryProfile`,
  `MaterialResolver`, `ResolvedCredentialMaterial`, `SecretEnv`, and
  `CredentialDelivery`.
- `crates/runx-runtime/src/adapters/cli_tool.rs` injects
  `CredentialDelivery.secret_env()` only at child process spawn and redacts
  captured stdout/stderr.
- `crates/runx-runtime/src/adapters/mcp/**` passes `SecretEnv` to MCP process
  spawn and redacts tool results.

Existing contract surfaces:
- `packages/contracts/src/schemas/credentials.ts` owns credential envelope and
  authority-proof shapes without raw material.
- `crates/runx-contracts/src/external_adapter.rs` has credential references and
  credential request frames, but no host-to-adapter delivery frame or delivery
  mode.
- `docs/security-authority-proof.md` bans raw tokens and records only
  `material_ref` hashes in public proof.

## Objectives

- Define a stable credential-delivery frame/envelope family in
  `runx-contracts` and `@runxhq/contracts`.
- Define delivery modes for v1. The default should be process environment
  injection because Rust already implements that safely for built-in adapters.
  File/socket/helper-process delivery remains future work unless a v1 consumer
  proves it is required.
- Define a broker response that carries only delivery handles or scoped secret
  material over trusted host-to-supervisor channels. Adapters must not request
  arbitrary credentials at runtime.
- Define delivery profiles: provider, auth mode, purpose, material roles, target
  env/file names, required/optional semantics, and redaction hints.
- Define redaction and non-leakage rules shared by cli-tool, MCP, external
  execution adapters, and outbox/provider adapters.
- Define receipt/proof observations: material is omitted; receipts may record
  credential refs, grant refs, provider, purpose, profile id, delivery mode, and
  material ref hash only.

## Scope

In scope:
- Contract schemas and Rust/TypeScript types for credential delivery requests,
  broker responses, delivery profiles, and redaction policy.
- Runtime mapping from admitted credential envelope + binding decision +
  material resolver to a delivery object.
- External execution-adapter host-to-process delivery semantics.
- Outbox/provider adapter delivery requirements where provider tokens are needed
  for side effects.
- Tests proving secrets do not enter receipts, authority proofs, stdout/stderr,
  metadata, or external adapter response observations.

Out of scope:
- Secret storage implementation, OAuth handshakes, hosted grant lifecycle, and
  BYO credential verification; owned by `byo-credential-foundations` and cloud
  auth specs.
- Provider-specific SDK packages.
- Source-event ingress protocol design.
- Replacing the existing authority proof or credential envelope shape.
- General-purpose auth resolver plugins.

## Dependencies

- `rust-adapter-credential-delivery` archived completed; provides the current
  Rust local implementation mechanics and non-leakage tests.
- `byo-credential-foundations`; owns storage, verification, and hosted material
  availability.
- `external-adapter-plugin-protocol-v1`; consumes this contract for external
  execution-adapter delivery and must not invent a separate credential channel.
- `skill-author-runtime-contract-v1`; the author-facing subprocess ABI must stay
  compatible with the same delivery primitive.
- `github-outbox-receipts`; outbox/provider side effects must use this primitive
  or a named blocker if provider credentials are required.
- `security-authority-proof.md`; public proof remains metadata-only and
  secret-free.

## Touchpoints

- `crates/runx-contracts/src/`
- `packages/contracts/src/schemas/`
- `schemas/credential-*.schema.json`
- `fixtures/contracts/credential-delivery/`
- `crates/runx-runtime/src/credentials.rs`
- `crates/runx-runtime/src/adapters/cli_tool.rs`
- `crates/runx-runtime/src/adapters/mcp/`
- `crates/runx-runtime/src/adapters/external_adapter.rs`
- `docs/security-authority-proof.md`
- `oss/.scafld/specs/active/external-adapter-plugin-protocol-v1.md`

## Risks

- If delivery is left per-protocol, cli-tool, external adapters, hosted runtimes,
  and outbox providers will each grow incompatible secret channels.
- If the contract is too rich, it becomes an auth plugin system or secret store.
- If v1 allows HTTP delivery without auth, retry, and idempotency semantics, it
  can leak secrets or double-use scoped credentials.
- If redaction is exact-string only, transformed or boundary-split secrets can
  leak through captured output. The contract must specify the minimum v1
  guarantee and the limits of that guarantee honestly.

## Acceptance

Profile: strict

Definition of done:
- [x] `dod1` Contract schemas exist for delivery profile, delivery request,
  broker response, and runtime delivery observation.
- [x] `dod2` Rust and TypeScript contract fixtures cross-validate and reject raw
  secret material in public frames.
- [x] `dod3` Runtime delivery uses the contract for cli-tool/MCP without
  weakening the existing `CredentialDelivery` secret channel.
- [ ] `dod4` External execution-adapter Phase 2 wiring consumes this contract
  instead of accepting arbitrary credential-request frames from the adapter.
- [ ] `dod5` Outbox/provider adapter specs either consume this contract or mark
  provider credentials as an explicit blocker.
- [ ] `dod6` Redaction tests cover stdout, stderr, metadata, response
  observations, receipt metadata, and truncation boundaries.

Validation:
- [ ] `v1` Scafld validates this spec.
  - Command: `scafld validate credential-broker-delivery-contract-v1 --json`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-22T00:45:00+10:00 returned
    `{"ok":true,"command":"validate","result":{"task_id":"credential-broker-delivery-contract-v1","path":"/Users/kam/dev/runx/runx/oss/.scafld/specs/drafts/credential-broker-delivery-contract-v1.md","valid":true,"errors":null}}`.
- [ ] `v2` Contract schema generation is fresh.
  - Command: `pnpm contracts:schemas:check`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-22T00:43:33+10:00 exited 0 after generating the four
    credential-delivery schemas.
- [ ] `v3` Rust contract fixtures pass.
  - Command:
    `cargo test --manifest-path crates/Cargo.toml -p runx-contracts credential_delivery`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-22T00:43:54+10:00 focused fixture command passed
    `credential_delivery_fixtures` and `schema_validation`.
- [ ] `v4` Runtime credential delivery tests pass.
  - Command:
    `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test credential_delivery --features cli-tool,mcp -- --nocapture`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-22T00:52:00+10:00 passed 9 focused tests, including
    contract-profile mapping, unsupported role rejection, empty material
    rejection, redact-before-truncate behavior, and MCP real process transport
    delivery/redaction.
- [ ] `v5` External adapter supervisor tests prove delivery or fail closed.
  - Command:
    `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features external-adapter,cli-tool external_adapter`
  - Expected kind: `exit_code_zero`
  - Status: pending
  - Evidence: none

## Phase 1: Contract Shape

Goal: freeze the shared credential-delivery wire vocabulary before external
adapter runtime wiring.

Status: complete for the process-env public contract surface
Dependencies: `rust-adapter-credential-delivery`,
`byo-credential-foundations`

Changes:
- [x] Add contract types for:
  - delivery profile;
  - credential delivery request from supervisor to broker;
  - broker response from broker to supervisor;
  - runtime delivery observation for receipts/proofs.
- [x] Declare v1 process-env delivery mode and reserve future modes explicitly.
- [x] Declare that public frames carry refs, hashes, provider, purpose, delivery
  mode, and profile ids only. Raw secret material is private to the trusted
  broker/supervisor channel.

Acceptance:
- Generated schemas and fixtures reject raw `access_token`, `refresh_token`,
  `api_key`, `password`, and `client_secret` fields outside the private delivery
  object.

Evidence:
- `packages/contracts/src/schemas/credential-delivery.ts`
- `packages/contracts/src/schemas/credential-delivery.test.ts`
- `crates/runx-contracts/src/credential_delivery.rs`
- `crates/runx-contracts/tests/credential_delivery_fixtures.rs`
- `fixtures/contracts/credential-delivery/*.json`
- `schemas/credential-delivery-*.schema.json`
- `pnpm vitest run packages/contracts/src/schemas/credential-delivery.test.ts packages/contracts/src/schemas/credentials.test.ts`
  passed 7 tests.
- `cargo test --manifest-path crates/Cargo.toml -p runx-contracts --test
  credential_delivery_fixtures --test schema_validation -- --nocapture` passed
  7 tests.
- `pnpm fixtures:contracts:keys` passed.

## Phase 2: Runtime Adoption

Goal: align existing Rust delivery mechanics to the contract.

Status: complete for built-in cli-tool/MCP process-env delivery
Dependencies: Phase 1

Changes:
- [x] Map `CredentialDeliveryProfile` and `CredentialDelivery` to/from the contract
  where appropriate without adding serialization to secret-bearing Rust types.
- [x] Reject empty material values before env injection.
- [x] Redact before truncation for cli-tool process output.
- [x] Add an MCP real-spawn credential delivery integration test, not only fixture
  transport coverage.

Acceptance:
- The existing local credential delivery behavior remains fail-closed and the
  archived non-leakage guarantees still pass.

Evidence:
- `crates/runx-runtime/src/credentials.rs` maps from
  `runx_contracts::CredentialDeliveryProfile`, rejects unsupported roles, and
  rejects empty material before env injection.
- `crates/runx-runtime/src/adapters/cli_tool.rs` now calls
  `CredentialDelivery::redact_bytes_to_string` so redaction occurs before final
  output truncation.
- `crates/runx-runtime/tests/credential_delivery.rs` passed 9 tests with
  `--features cli-tool,mcp`.

## Phase 3: External Adapter And Outbox Consumption

Goal: make cross-boundary consumers use the same primitive.

Status: pending
Dependencies: Phase 2, `external-adapter-plugin-protocol-v1`

Changes:
- Replace or narrow external adapter credential-request handling so adapters
  receive only host-delivered credential refs/handles/material through the
  approved delivery mode.
- External adapter supervisor injects process-env delivery after audited scoped
  env and redacts stdout/stderr/response observations.
- Outbox/provider adapter specs declare whether provider credentials are
  delivered through this primitive or are not in scope.

Acceptance:
- No protocol introduces an independent secret-delivery path.

## Rollback

If the shared contract cannot cover a consumer, keep that consumer blocked or
give it a named sibling credential-delivery extension. Do not smuggle raw secret
material through existing invocation metadata, adapter env fields, receipts, or
provider-specific JSON blobs.

## Review

Review must reject:
- raw secret fields in public contracts, receipts, authority proofs, or adapter
  observations;
- adapter-initiated arbitrary credential requests;
- protocol-specific secret channels that bypass the shared broker/delivery
  primitive;
- widening this spec into OAuth, BYO storage, source ingress, or general auth
  plugin design.

## Origin

User architecture review on 2026-05-22: after the external execution-adapter
scope was corrected, the remaining cross-cutting gap is credential
broker/delivery. It must be a shared primitive consumed by cli-tool, external
adapters, and outbox/provider side effects rather than a per-protocol
afterthought.
