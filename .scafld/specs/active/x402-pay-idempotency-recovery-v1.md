---
spec_version: '2.0'
task_id: x402-pay-idempotency-recovery-v1
created: '2026-05-21T00:00:00Z'
updated: '2026-05-22T00:55:00+10:00'
status: active
harden_status: not_run
size: medium
risk_level: high
---

# x402-pay idempotency recovery v1

## Current State

Status: complete
Current phase: phase1 runtime complete; fixture promotion complete
Next: keep runtime and harness fixtures as regression coverage
Reason: Durable payment state primitives now exist and have focused runtime
coverage. P1.7 is covered at runtime by replaying a sealed idempotency entry
from replay-safe stored outputs without a second rail invocation. P1.9 remains
covered by proving a consumed spend capability with a new idempotency key denies
before rail. P1.11 is covered at runtime by escalating an in-flight rail
mutation without issuing a second rail mutation. All three cases now also have
standalone `x402-pay-idempotency-*` harness fixtures.
Blockers: none for P1.7/P1.9/P1.11 runtime and fixture coverage.
Allowed follow-up command: `scafld validate x402-pay-idempotency-recovery-v1`
Latest runner update: 2026-05-22T00:55:00+10:00 promoted the standalone
fixture matrix and validated it through the Rust harness.
Review gate: not_started

## Summary

Turn the remaining x402 Phase 1 idempotency and recovery eventualities into an
executable scafld contract:

- P1.7: replaying the same idempotency key returns the first sealed receipt and
  does not execute a second mock spend.
- P1.9: reusing the same single-use spend capability is denied by core, not by
  the mock rail.
- P1.11: a crash or abort after a partial mock rail mutation is recoverable by
  idempotency key and either seals the existing mutation or escalates with a
  typed recovery state.

The original "no observable payment state" blocker is lifted for focused Rust
state tests, same-key replay is executable at the runtime layer, partial rail
mutation recovery has a fail-closed escalation path, and the runnable fixture
set is promoted under `x402-pay-idempotency-*`.

## Context

CWD: `.`

Packages:
- `crates/runx-core`
- `crates/runx-runtime`
- `fixtures/graphs/payment`
- `fixtures/harness`
- `fixtures/skills/payment-fulfill`

Files impacted:
- `.scafld/specs/active/x402-pay-idempotency-recovery-v1.md`
- `fixtures/harness/x402-pay-idempotency-*.yaml`
- `fixtures/graphs/payment/x402-pay-idempotency-*.yaml`
- `fixtures/skills/x402-pay-idempotency-*/SKILL.md`

Invariants:
- Do not depend on a native `runx x402-pay`, `runx receipts`, or `runx ledger`
  command; current observable surfaces are `runx harness`, receipt output, and
  `runx history`.
- Do not touch `.scafld/specs/drafts/rust-nitrosend-dogfood.md`.
- Do not edit `crates/runx-cli/tests/x402_native_dogfood.rs` or
  `tests/x402-pay-dogfood-mock.test.ts` unless coordination confirms no other
  x402 worker owns them.
- New fixtures, when promoted, must live under clearly named
  `x402-pay-idempotency-*` paths.

Related docs:
- `.scafld/specs/archive/2026-05/x402-pay-dogfood-v1.md`
- `.scafld/specs/archive/2026-05/x402-pay-phase1-mock-scenario-fixtures-v1.md`

## Blocker Evidence

The current implementation now has durable payment state, runtime
replay/recovery behavior, and promoted fixture-backed coverage:

- `crates/runx-runtime/src/payment_state.rs:287` exposes persisted consumed
  spend-capability lookup, and `crates/runx-runtime/src/payment_state.rs:301`
  exposes persisted idempotency lookup.
- `crates/runx-runtime/src/payment_state.rs:315` persists payment step state,
  including consumed spend capability records, sealed idempotency entries, and
  mock rail mutation records.
- `crates/runx-runtime/src/execution/runner/steps.rs:96` calls
  `persist_payment_step_state` after the spend step receipt is built.
- `crates/runx-runtime/src/execution/runner/authority.rs:97` injects persisted
  consumed capability refs into core admission, so P1.9 no longer depends only
  on fixture-seeded `consumed_spend_capability_refs`.
- `crates/runx-runtime/src/payment_state.rs` stores replay-safe sealed outputs
  plus the original receipt timestamp and digest for idempotency replay. The
  stored outputs remove rail session material before persistence.
- `crates/runx-runtime/src/execution/runner/authority.rs` now detects sealed
  idempotency entries before persisted spend-consumption admission, revalidates
  the current authority shape without treating the capability as a fresh spend,
  and returns replay material to the runner.
- `crates/runx-runtime/src/execution/runner/steps.rs` short-circuits sealed
  idempotency replay before adapter invocation, rebuilds the original payment
  step receipt from stored material, and fails closed if receipt id, digest, or
  typed rail proof do not match the persisted entry.
- `crates/runx-runtime/tests/payment_execution.rs` proves both P1.7 runtime
  replay with no second `pay-fulfill-rail` call and P1.9 persisted consumed
  capability denial when the second run uses a new idempotency key.
- Partial rail state can be persisted as `in_flight`, and the runner now
  escalates that state by idempotency key before any second rail mutation is
  allowed. P1.11 runtime semantics are covered.
- `fixtures/harness/x402-pay-idempotency-replay.yaml` proves P1.7 fixture
  replay with one rail invocation.
- `fixtures/harness/x402-pay-idempotency-capability-reuse.yaml` proves P1.9
  fixture denial before a second rail invocation.
- `fixtures/harness/x402-pay-idempotency-crash-recovery.yaml` proves P1.11
  fixture recovery escalation before retrying the rail.

## Objectives

- Specify and execute the fixture matrix for P1.7, P1.9, and P1.11.
- Keep all fixture names under `x402-pay-idempotency-*`.

## Scope

- Runtime state and runner replay work for P1.7/P1.9.
- Fixture promotion for P1.7/P1.9/P1.11 after the corresponding runtime
  behavior is satisfied.
- No shared x402 dogfood tests are edited by this spec.

## Dependencies

- Durable idempotency index keyed by rail family, counterparty or grant, and
  idempotency key.
- Durable spend-capability consumption record keyed by capability ref and
  linked to the sealing receipt or recovery state.
- Durable mock rail mutation record with at least: idempotency key, rail,
  amount, currency, counterparty, mutation status, proof ref when known, and
  recovery classification.

## Assumptions

- The mock rail remains deterministic and local.
- Recovery may return either a sealed receipt or a governed escalation, but it
  must not silently execute an additional spend.
- The first implementation can use file-backed state as long as it survives
  process restart within a harness run and is observable by tests.

## Touchpoints

- `.scafld/specs/active/x402-pay-idempotency-recovery-v1.md`
- `crates/runx-runtime/tests/payment_execution.rs`
- `crates/runx-runtime/tests/payment_state.rs`
- `fixtures/harness/x402-pay-idempotency-replay.yaml`
- `fixtures/harness/x402-pay-idempotency-capability-reuse.yaml`
- `fixtures/harness/x402-pay-idempotency-crash-recovery.yaml`
- `fixtures/graphs/payment/x402-pay-idempotency-*.yaml`

## Risks

- Description: Static fixtures could falsely pass by duplicating hard-coded
  inputs.
  Mitigation: require an observable persisted state delta and a no-second-spend
  assertion before adding fixtures.
- Description: Recovery could be confused with retry.
  Mitigation: require recovery classification from persisted rail state before a
  second rail execution is allowed.

## Acceptance

Profile: strict

Definition of done:
- [x] `dod1` P1.7 fixture proves same idempotency key returns the original
  receipt and the mock rail execution count stays one.
- [x] `dod2` P1.9 fixture proves the second use of a consumed spend capability
  is rejected by core from persisted state.
- [x] `dod3` P1.11 fixture proves recovery by idempotency key from a partial
  mock rail mutation.
- [x] `dod4` All new fixture paths are named `x402-pay-idempotency-*`.

Validation:
- [x] `v1` spec - This scafld spec validates.
  - Command: `scafld validate x402-pay-idempotency-recovery-v1`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0 on 2026-05-21T13:05:33Z
  - Source event: local
- [x] `v2` state-layer - Durable payment state has executable coverage.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test payment_state`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: `crates/runx-runtime/src/payment_state.rs` now exposes typed
    idempotency lookup, consumed spend capability lookup, and mock rail mutation
    persistence. `crates/runx-runtime/tests/payment_state.rs` covers sealed
    step-state persistence through public lookup helpers, first-record-wins
    idempotency persistence, consumed capability lookup, and partial rail
    mutation persistence without exposing a sealed replay entry. The command
    passed on 2026-05-21T13:05:33Z with 7 tests.
  - Source event: local
- [x] `v3` fixture - Idempotency replay fixture passes.
  - Command: `runx harness fixtures/harness/x402-pay-idempotency-replay.yaml`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: native CLI `cargo run --quiet --manifest-path crates/Cargo.toml -p runx-cli -- harness fixtures/harness/x402-pay-idempotency-replay.yaml --json`
    exited 0 on 2026-05-22T00:55:00+10:00 with a closed graph receipt. The
    focused Rust harness test also proves the sealed fulfill receipt is
    replayed and rail invocation count stays one.
  - Source event: local
- [x] `v4` fixture - Spend capability reuse fixture passes.
  - Command: `runx harness fixtures/harness/x402-pay-idempotency-capability-reuse.yaml`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: native CLI `cargo run --quiet --manifest-path crates/Cargo.toml -p runx-cli -- harness fixtures/harness/x402-pay-idempotency-capability-reuse.yaml --json`
    exited 0 on 2026-05-22T00:55:00+10:00 with a blocked graph receipt and
    reason `x402_idempotency_capability_reuse_blocked`. The focused Rust
    harness test also proves the second graph run is denied before a second rail
    mutation.
  - Source event: local
- [x] `v5` fixture - Partial mutation recovery fixture passes.
  - Command: `runx harness fixtures/harness/x402-pay-idempotency-crash-recovery.yaml`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: native CLI `cargo run --quiet --manifest-path crates/Cargo.toml -p runx-cli -- harness fixtures/harness/x402-pay-idempotency-crash-recovery.yaml --json`
    exited 0 on 2026-05-22T00:55:00+10:00 with a deferred graph receipt and
    reason `x402_idempotency_recovery_escalated`. The focused Rust harness test
    also proves the second graph run escalates from persisted partial rail
    mutation state before retrying the rail.
  - Source event: local
- [x] `v6` runtime P1.9 - Reusing the same single-use spend capability is
  denied from persisted state before a second rail call.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test payment_execution x402_paid_echo_reused_spend_capability_with_new_idempotency_denied_from_persisted_state_before_second_rail -- --nocapture`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0 on 2026-05-21T14:09:12Z; the first paid echo run
    sealed a spend under `payment:paid-echo-001`, the second run used
    `payment:paid-echo-002`, returned `AuthorityDenied { verb: Spend, step_id:
    fulfill }` with an already-consumed reason, and the recorded
    `pay-fulfill-rail` invocation count stayed at one.
  - Source event: local
- [x] `v7` runtime P1.7 - Replaying the same sealed idempotency key returns the
  sealed payment output and does not execute a second rail call.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test payment_execution x402_paid_echo_replays_sealed_idempotency_without_second_rail -- --nocapture`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0 on 2026-05-21T14:09:12Z; the second paid echo run
    succeeded, the replayed fulfill receipt id and digest matched the first run,
    the paid echo step received the stored proof, the `pay-fulfill-rail`
    invocation count stayed at one, and persisted replay state did not contain
    the rail session material reference.
  - Source event: local
- [x] `v8` runtime regression - Payment execution suite still passes with replay
  and consumed-capability denial both enabled.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test payment_execution -- --nocapture`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0 on 2026-05-22T00:55:00+10:00; 18 tests passed.
  - Source event: local
- [x] `v9` runtime P1.11 - In-flight rail mutation recovery escalates without
  issuing a second rail mutation.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test payment_execution x402_paid_echo_partial_mutation_escalates_without_second_rail -- --nocapture`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0 on 2026-05-21T14:17:26Z; the first paid echo run
    recorded a partial rail mutation, the second run returned
    `AuthorityDenied { verb: Spend, step_id: fulfill }` with a recovery
    escalation reason before adapter invocation, `pay-fulfill-rail` invocation
    count stayed at one, and persisted rail mutation state moved to
    `escalated`.
  - Source event: local
- [x] `v10` compatibility - v2 payment state opens fail-closed after v3 replay
  fields were added.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test payment_state -- --nocapture`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0 on 2026-05-21T14:17:26Z; 8 tests passed,
    including v2 state loading that preserves consumed capability state while
    dropping legacy sealed idempotency entries that lack replay-safe outputs.
  - Source event: local

## Phase 1: State Layer Contract

Goal: expose the durable state needed for P1.7, P1.9, and P1.11.

Status: complete
Dependencies: none

Changes:
- `crates/runx-runtime/src/payment_state.rs` - persists idempotency entries,
  spend capability consumption, and mock rail mutation state.
- `crates/runx-runtime/tests/payment_state.rs` - covers the durable state
  semantics available before fixture-level replay/recovery.

Acceptance:
- [x] `ac1_1` state - A sealed payment receipt can be looked up by
  idempotency key after process restart.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test payment_state persists_sealed_payment_step_state_for_replay_and_reuse_lookups`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: public lookup returns the sealed receipt ref from file-backed
    state resolved via the receipt directory fallback.
  - Source event: local
- [x] `ac1_2` state - A consumed spend capability ref is rejected when reused
  without requiring the fixture to seed `consumed_spend_capability_refs`.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test payment_execution x402_paid_echo_reused_spend_capability_with_new_idempotency_denied_from_persisted_state_before_second_rail -- --nocapture`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: the runtime injected persisted consumed capability state into
    core admission and denied before the second rail invocation.
  - Source event: local
- [x] `ac1_4` state - A sealed idempotency entry can be replayed from stored
  outputs without issuing a second rail mutation.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test payment_execution x402_paid_echo_replays_sealed_idempotency_without_second_rail -- --nocapture`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: the runner short-circuited before adapter invocation for
    `pay-fulfill-rail`, rebuilt the first fulfill receipt id/digest, and
    forwarded the stored proof to the paid echo step.
  - Source event: local
- [x] `ac1_3` state - A partial mock rail mutation is recoverable by
  idempotency key without issuing a second rail mutation.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test payment_execution x402_paid_echo_partial_mutation_escalates_without_second_rail -- --nocapture`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: the runner observes the existing in-flight mutation by idempotency
    key, validates the current authority shape, escalates typed rail mutation
    state, and denies before a second `pay-fulfill-rail` invocation.
  - Source event: local

## Phase 2: Executable Fixtures

Goal: add the three fixture-backed eventualities now that Phase 1 runtime
semantics are executable.

Status: complete
Dependencies:
- phase1

Changes:
- `fixtures/harness/x402-pay-idempotency-replay.yaml` -
  executes one payment, replays the same idempotency key, and asserts the first
  receipt is returned with one rail mutation.
- `fixtures/harness/x402-pay-idempotency-capability-reuse.yaml` - executes one
  payment, attempts a second spend with the same capability ref, and asserts a
  core denial before rail execution.
- `fixtures/harness/x402-pay-idempotency-crash-recovery.yaml` - simulates a
  crash after partial mock rail mutation, invokes recovery by idempotency key,
  and asserts escalation before a second rail execution.

Acceptance:
- [x] `ac2_1` fixture - P1.7 has a runnable fixture.
  - Command: `test -f fixtures/harness/x402-pay-idempotency-replay.yaml`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: file exists and passed through the Rust harness x402 idempotency
    sequence test on 2026-05-22T00:42:00+10:00.
  - Source event: local
- [x] `ac2_2` fixture - P1.9 has a runnable fixture.
  - Command: `test -f fixtures/harness/x402-pay-idempotency-capability-reuse.yaml`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: file exists and passed through the Rust harness x402 idempotency
    sequence test on 2026-05-22T00:42:00+10:00.
  - Source event: local
- [x] `ac2_3` fixture - P1.11 has a runnable fixture.
  - Command: `test -f fixtures/harness/x402-pay-idempotency-crash-recovery.yaml`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: file exists and passed through the Rust harness x402 idempotency
    sequence test on 2026-05-22T00:42:00+10:00.
  - Source event: local

## Rollback

Strategy: per_phase

Commands:
- `rm .scafld/specs/active/x402-pay-idempotency-recovery-v1.md`

## Review

Status: not_started
Verdict: none
Timestamp: none
Review rounds: none
Reviewer mode: none
Reviewer session: none
Round status: none
Override applied: none
Override reason: none
Override confirmed at: none
Reviewed head: none
Reviewed dirty: none
Reviewed diff: none
Blocking count: none
Non-blocking count: none

Findings:
- none

Passes:
- none

## Self Eval

Status: complete
Completeness: durable state semantics, runtime replay, runtime recovery
escalation, and standalone fixture promotion covered
Architecture fidelity: current harness and runtime state surfaces respected
Spec alignment: P1.7, P1.9, and P1.11 mapped directly
Validation depth: focused Rust runtime state tests plus full payment execution
suite
Total: complete
Second pass performed: yes

Notes:
Runtime replay, fail-closed recovery escalation, and standalone harness
fixtures are now executable.

Improvements:
- Keep these fixtures in the payment regression suite when x402 provider
  projectors evolve.

## Deviations

- The harness promotion is intentionally sequence-aware because each fixture
  needs two graph executions over one shared payment-state file.

## Metadata

Estimated effort hours: 4
Actual effort hours: 1
AI model: gpt-5-codex
React cycles: 0

Tags:
- x402
- payments
- idempotency
- recovery

## Origin

Source:
- Worker E OSS x402 idempotency/recovery spec lane

Repo:
- `/Users/kam/dev/runx/runx/oss`

Git:
- dirty worktree; unrelated `rust-nitrosend-dogfood.md` modification preserved

Sync:
- none

Supersession:
- none

## Harden Rounds

- none

## Planning Log

- 2026-05-21T00:00:00Z: Filed blocked spec for P1.7/P1.9/P1.11 after code
  evidence showed missing observable persisted payment state.
- 2026-05-21T13:05:33Z: Confirmed durable payment state primitives exist,
  added focused runtime state tests, and kept fixtures blocked on runner
  replay/recovery behavior.
- 2026-05-21T13:17:07Z: Added runtime P1.9 coverage proving persisted consumed
  spend capability state denies a second paid echo run before a second rail
  invocation.
- 2026-05-21T14:09:12Z: Added replay-safe sealed output persistence, runner
  idempotency replay before rail invocation, runtime P1.7 coverage, and updated
  P1.9 to prove a new idempotency key with the consumed spend capability still
  denies before rail.
- 2026-05-21T14:17:26Z: Added fail-closed in-flight rail mutation recovery
  escalation before a second rail invocation, v2 payment-state compatibility,
  and runtime P1.11 coverage.
- 2026-05-22T00:55:00+10:00: Promoted P1.7/P1.9/P1.11 into standalone
  `x402-pay-idempotency-*` harness fixtures and validated them through the Rust
  harness.
