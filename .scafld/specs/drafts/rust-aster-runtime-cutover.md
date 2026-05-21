---
spec_version: '2.0'
task_id: rust-aster-runtime-cutover
created: '2026-05-18T00:00:00Z'
updated: '2026-05-21T22:52:32+10:00'
status: draft
harden_status: not_run
size: large
risk_level: high
---

# Rust aster runtime cutover

## Current State

Status: draft
Current phase: external-runtime-fixture plus external Aster dogfood smoke
Next: cloud-tree binding pass against the sibling `../cloud` workspace, without
guessing cloud package internals from the OSS crate
Reason: the OSS-local external Aster agent-step replay fixture is grounded in
the Aster repo's current Rust bridge scripts, and the live Aster checkout now
passes its local Rust-binary proving-ground smoke.
Blockers: cloud package binding is still unverified by this draft because
`cloud/**` is not part of the OSS crate checkout. The full workspace contains a
sibling cloud repo, but the cloud binding needs its own inspected pass; this
draft does not settle `agent-runner` or choose between hosted HTTP, CLI JSON,
service/FFI, or external adapter/plugin protocol boundaries.
Allowed follow-up command: none during this refresh; do not run
`scafld harden rust-aster-runtime-cutover`.
Latest runner update: 2026-05-21T22:52:32+10:00 Aster checkout audit and pin
bump evidence remains: local Aster repo `/Users/kam/dev/runx/aster` is clean at
`e084a31d` after pushing `chore(runx): bump dogfood pin`; its dogfood pin now
targets pushed Runx OSS SHA `19e063666b3a6aa4f390c618dec84f5d59cd558d` from
parent workspace commit `3476c16`. Aster validation passed `npm run check`,
targeted bridge/pin tests, `node scripts/runx-checkout-pin.mjs resolve`, and
`git diff --check`. Hosted Aster dogfood now fetches the same pushed Runx SHA
used for local validation evidence. This refresh also aligns cloud-binding
language with `ts-extension-survivorship-boundary` and
`external-adapter-plugin-protocol-v1` so this draft cannot be read as settling
`agent-runner` while the cloud boundary is still open.
Review gate: not_started

## Summary

Plan the Aster runtime cutover from the local OSS state plus the adjacent Aster
checkout that is actually available. The OSS crate checkout does not include
`cloud/**`, so this spec cannot claim verified cloud package paths, UI paths,
hosted agent adapter files, or cloud DB approval routing. The full workspace
does include a sibling cloud repo, but those bindings stay deferred until a
dedicated pass inspects that tree and records exact paths. This draft therefore
does not settle the cloud `agent-runner` binding for the runtime-local/adapters
sunset.

Current local facts:

- `crates/runx-runtime/src/runtime_http.rs` is the hosted transport boundary
  visible in this checkout. It defines `HostedHttpClient`, `HostedTransport`,
  request and response types, header validation, reqwest/rustls-backed
  transport under the `async-http` feature, redirect suppression, bounded
  request/connect timeouts, and redacted debug behavior.
- Aster contract types exist in `crates/runx-contracts/src/aster.rs`.
- The contracts crate exports Aster control objects from
  `crates/runx-contracts/src/lib.rs`.
- A structural Aster control fixture exists at
  `fixtures/contracts/aster-control/public-feed-proof.json`.
- A runtime external fixture now exists for the local Aster Rust bridge shape:
  `fixtures/external/aster/agent-step/rust-bridge-sealed-skill.yaml`.
- A focused runtime test now exists at
  `crates/runx-runtime/tests/external/aster_agent_step.rs`, wired through
  `crates/runx-runtime/tests/external.rs`.
- The local checkout has no `cloud/` directory and no
  `crates/runx-runtime/src/cloud_client.rs`.
- The readable Aster checkout at `/Users/kam/dev/runx/aster` currently routes
  Rust execution through `scripts/aster-core.mjs` into
  `scripts/runx-agent-bridge.mjs`; the accepted terminal skill report is
  `runx.skill_run.v1` with `status: "sealed"` and a stored
  `runx.harness_receipt.v1` receipt id.
- The Aster checkout dogfoods the Rust binary directly for harness execution;
  it does not invoke a JS/npm Runx CLI bridge for the proving-ground path.
- `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test external
  aster_agent_step` passes for the new fixture replay.

The cutover remains preservation-oriented: Aster should consume the Rust
runtime through a documented boundary and canonical contracts, but this draft
must not invent a cloud binding, claim an agent-step runtime fixture before
those files exist, or imply that custom adapter/plugin authors must link into
Rust. If Aster needs custom userland integration code, that belongs behind the
language-neutral external adapter/plugin protocol under Rust supervision rather
than behind `@runxhq/runtime-local`.

## Context

CWD: `.` (runx OSS workspace)

Relevant existing local surfaces:

- `crates/runx-runtime/src/runtime_http.rs`
- `crates/runx-contracts/src/aster.rs`
- `crates/runx-contracts/src/lib.rs`
- `fixtures/contracts/aster-control/public-feed-proof.json`
- `crates/runx-contracts/tests/aster_control_fixtures.rs`
- `fixtures/operational-policy/nitrosend-like.json` as the current
  operational-policy readback proof point, not as an Aster runtime fixture.
- `.scafld/specs/drafts/runx-target-repo-runners.md`
- `.scafld/specs/drafts/runx-post-merge-closure-observer.md`

Surfaces not present in this checkout:

- `cloud/packages/**`
- `cloud/packages/agent-runner/**`
- `cloud/packages/api/**`
- `cloud/packages/db/**`
- `cloud/packages/receipts-store/**`
- `cloud/packages/ui/**`
- `crates/runx-runtime/src/cloud_client.rs`
- `cloud/**`

## Invariants

- Cloud binding is deferred until a checkout with the cloud tree is available.
  This spec may name the required boundary, but it must not assert verified
  cloud implementation paths in the OSS-only checkout.
- Cloud `agent-runner` binding is an open follow-up, not a settled Aster
  cutover fact. The later pass must choose an allowed stable boundary such as
  hosted HTTP, CLI JSON, service/FFI, or the external adapter/plugin protocol,
  and must not preserve a runtime-local fallback.
- Aster control objects use the existing `runx-contracts::aster` shapes. Do not
  create parallel Aster JSON shapes for target, opportunity, selection,
  reflection, skill binding, feed entry, or transition records.
- Runtime execution artifacts stay canonical harness, decision, act,
  verification/proof, and sealed `runx.harness_receipt.v1` objects.
- Aster must not read receipts through private local file paths in public or
  hosted projections; receipt access goes through runtime/store APIs or a
  documented hosted boundary.
- `runtime_http.rs` is the current local hosted transport boundary. Any future
  cloud binding should either use this boundary through the public
  connect/registry re-exports or explicitly replace it in a separate reviewed
  change.
- External adapter/plugin use, if needed by Aster or cloud agent integrations,
  follows `external-adapter-plugin-protocol-v1`; this spec must not require
  provider-specific adapter code to become a Rust crate.
- No legacy/compat outcome, effect, verification proof alias, or Aster-only terminal
  packet is introduced.

## Objectives

- Preserve the Aster contract surface already present in
  `crates/runx-contracts/src/aster.rs` and its fixture coverage.
- Define the runtime external fixture that is missing today:
  `fixtures/external/aster/agent-step/**`.
- Add a Rust runtime replay test only after the fixture exists:
  `crates/runx-runtime/tests/external/aster_agent_step.rs`.
- Use `runtime_http.rs` as the locally verified hosted transport boundary for
  any OSS-side runtime-to-host interaction.
- Defer cloud package binding details until the cloud tree is available.
- Ensure Aster-run issue-to-PR and post-merge paths use
  `runx-target-repo-runners` and `runx-post-merge-closure-observer` when those
  contracts exist, with final state represented as sealed closure/proof
  receipts.

## Scope

In scope:

- OSS-local plan for Aster contract preservation.
- External Aster runtime fixture definition.
- Hosted boundary notes grounded in `runtime_http.rs`.
- Dependency sequencing for target-runner and post-merge observer flows.

Out of scope:

- Editing or verifying `cloud/**` paths in this checkout.
- Implementing the cloud binding shim.
- Aster UI, feed curation, selector product behavior, or brand work.
- Scafld hardening in this refresh.
- Legacy/compat execution artifact shapes.

## Dependencies

- `runx-contract-spine-hard-cutover`.
- `rust-runtime-skeleton`.
- `rust-runtime-skill-execution`.
- `rust-approval-gate-parity` for any hosted approval gates that Aster consumes.
- `rust-runtime-receipt-path-discovery`,
  `rust-receipt-tree-resolution`, and `rust-receipt-proof-verification`.
- `runx-operational-policy-config` for policy/admin readback.
- `runx-target-repo-runners` for Aster-scheduled source-to-target PR flows.
- `runx-post-merge-closure-observer` for final closure/proof observation and
  source-thread updates.
- `ts-extension-survivorship-boundary` for the rule that TypeScript may survive
  as cloud/product/helper code over stable protocols but not as trusted local
  runtime execution.
- `external-adapter-plugin-protocol-v1` for any Aster or cloud custom
  adapter/plugin boundary that needs no-Rust-required userland code.
- `embedded-sdk-migration-story` for embedded SDK and cloud runtime-local
  consumer disposition.
- A future cloud-tree binding pass that can inspect the real `cloud/**`
  implementation.

## Acceptance Criteria

- [x] Existing Aster contract fixture coverage remains green for
  `fixtures/contracts/aster-control/public-feed-proof.json`.
- [x] The runtime external fixture
  `fixtures/external/aster/agent-step/**` exists before any Aster runtime
  replay test is claimed.
- [x] The replay test
  `crates/runx-runtime/tests/external/aster_agent_step.rs` is added only after
  the external fixture exists.
- [x] The OSS-hosted boundary is documented against
  `crates/runx-runtime/src/runtime_http.rs` or a reviewed replacement.
- [x] Cloud binding details are marked deferred until `cloud/**` is available
  locally; no acceptance depends on absent cloud paths.
- [x] Cloud `agent-runner` binding mode is not claimed as settled by this draft.
- [x] Aster contract and runtime artifacts use harness receipt closure and
  `proof.verification`, not retired peer terminal artifacts or legacy
  outcome/effect packet fields.
- [ ] Aster final publication and issue-to-PR completion, once implemented, use
  sealed harness receipt closure/proof through the reusable observer/runner
  specs rather than Aster-only terminal packets.

## Validation Commands

Current local discovery/guard commands:

```sh
test ! -d cloud
test -f crates/runx-runtime/src/runtime_http.rs
test -f crates/runx-contracts/src/aster.rs
test -f fixtures/contracts/aster-control/public-feed-proof.json
test -f fixtures/external/aster/agent-step/rust-bridge-sealed-skill.yaml
test -f crates/runx-runtime/tests/external/aster_agent_step.rs
cargo test --manifest-path crates/Cargo.toml -p runx-contracts aster
cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test external aster_agent_step
! rg -n "runx\\.issue_to_pr_outcome\\.v1|issue_to_pr_outcome|verification[_-]report|target[_-]?effect|\"effect\"\\s*:" crates/runx-contracts/src/aster.rs fixtures/contracts/aster-control
! rg -n "runId|receiptId|issue_to_pr_outcome|verification[_-]report|verificationReport|target[_-]?effect|\"effect\"\\s*:|\"outcome\"\\s*:|/Users/kam" fixtures/external/aster/agent-step/rust-bridge-sealed-skill.yaml
git diff --check -- .scafld/specs/drafts/rust-aster-runtime-cutover.md
```

Latest local validation:

```sh
cargo test --manifest-path crates/Cargo.toml -p runx-contracts aster
# passed: aster_control_fixture_roundtrips

cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test external aster_agent_step
# passed: 2 tests

ruby -ryaml -e 'YAML.load_file("fixtures/external/aster/agent-step/rust-bridge-sealed-skill.yaml"); puts "yaml ok"'
# passed: yaml ok

! rg -n "runId|receiptId|issue_to_pr_outcome|verification[_-]report|verificationReport|target[_-]?effect|\"effect\"\\s*:|\"outcome\"\\s*:|/Users/kam" fixtures/external/aster/agent-step/rust-bridge-sealed-skill.yaml
# passed: no matches
```

2026-05-21 Aster dogfood validation:

```sh
cd /Users/kam/dev/runx/aster
npm run check
# passed: aster check passed

node --test scripts/runx-agent-bridge.test.mjs scripts/run-issue-triage-workers.test.mjs scripts/promote-aster-state.test.mjs
# passed: 28 tests

/Users/kam/dev/runx/runx/oss/crates/target/debug/runx --version
# passed: runx-cli 0.1.0

RUNX_ROOT=/Users/kam/dev/runx/runx/oss ARTIFACT_DIR="$(mktemp -d /tmp/aster-proving-ground.XXXXXX)" PROVING_GROUND_PROFILE=minimal bash scripts/proving-ground.sh
node scripts/summarize-proving-ground.mjs "$ARTIFACT_DIR"
# passed: echo-skill and sequential-graph produced sealed runx.harness_receipt.v1 receipts
```

## Rollback And Repair

- If cloud binding assumptions are wrong, repair the cloud binding spec after
  inspecting a checkout that contains `cloud/**`; do not encode guessed cloud
  paths in this OSS-only spec.
- If cloud or Aster integration needs custom adapter/plugin code, route it
  through `external-adapter-plugin-protocol-v1` or keep the binding blocked; do
  not revive `@runxhq/runtime-local` or force provider code into Rust.
- If the external runtime fixture is missing, keep Aster cutover blocked rather
  than treating the Aster control contract fixture as runtime execution proof.
- If a future binding bypasses `runtime_http.rs`, require an explicit reviewed
  replacement boundary and update this spec.
- If retired artifact fields appear in Aster fixtures or runtime output, repair
  the producer and expected sealed receipts. Do not add compatibility shims.

## Open Questions

- Which concrete cloud binding mode wins once the cloud tree is available:
  hosted HTTP, subprocess JSON over `runx-cli`, `runx-runtime-service`/FFI, the
  external adapter/plugin protocol, or another reviewed stable boundary.
- Whether `cloud/packages/agent-runner/**` needs the external adapter/plugin
  protocol for hosted custom adapter behavior or can stay on a hosted HTTP
  boundary with generated contracts.
- Where hosted approval routing lives in the cloud tree after the Aster v1 reset
  work is available for inspection.
- Whether Aster needs a dedicated runtime fixture generator or can share the
  generic hosted fixture machinery once that exists.
