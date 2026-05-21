---
spec_version: '2.0'
task_id: skill-author-runtime-contract-v1
created: '2026-05-21T12:19:24Z'
updated: '2026-05-21T13:50:21Z'
status: completed
harden_status: not_run
size: medium
risk_level: high
---

# Skill author runtime contract v1

## Current State

Status: completed
Current phase: review-ready
Next: review
Reason: the author-visible subprocess ABI now matches the TypeScript adapter
for `RUNX_CWD`, `RUNX_INPUTS_JSON` versus `RUNX_INPUTS_PATH`, per-input env
caps, env-name normalization, source cwd resolution, and cwd-policy fail-closed
behavior in Rust. Rust now also drains stdout/stderr while the child runs and
has direct-child timeout coverage, so large author output no longer deadlocks.
Unix process-group timeout cleanup is now implemented for Rust cli-tool
children; non-Unix keeps the direct-child timeout fallback. The stable ABI is
documented in `docs/skill-author-runtime-contract.md`, and
`pnpm fixtures:skill-author-runtime:check` now runs the same executable fixture
entrypoint through the TypeScript adapter and Rust runtime. Completion was
reconfirmed on 2026-05-21T13:50:21Z by passing `scafld validate
skill-author-runtime-contract-v1 --json` and
`pnpm fixtures:skill-author-runtime:check`.
Allowed follow-up command: `scafld handoff skill-author-runtime-contract-v1`
Latest runner update: 2026-05-21T23:55:00+10:00 implemented concurrent
stdout/stderr draining in `crates/runx-runtime/src/adapters/cli_tool.rs` and
expanded `crates/runx-runtime/tests/cli_tool_contract.rs` to cover large stdout
and timeout behavior. Follow-up fixed workspace-root fallback semantics for
MCP server skill execution and expanded the Rust contract test suite to 12
cases. Focused Rust tests, TypeScript cli-tool tests, clippy, style, fmt,
whitespace, and spec validation passed.
This follow-up starts Unix cli-tool subprocesses in a new process group,
terminates the group on timeout, and expands the Rust contract suite to 13
cases with descendant cleanup coverage.
Phase 3 follow-up: 2026-05-21T15:55:00Z added
`fixtures/skill-author-runtime/`, a TypeScript fixture test, a Rust fixture
test, and the `fixtures:skill-author-runtime:check` script. The shared fixture
suite covers env delivery, stdin, cwd policy, large input spill, large output
drain/truncation, and timeout descendant cleanup across both runtimes.
Review gate: ready

## Summary

Define and test the v1 contract a skill author can rely on when runx executes a
subprocess skill. The contract must be runtime-neutral: the same `run.js` should
observe the same inputs, environment, cwd, stdio, timeout, output, and receipt
semantics whether launched by the TypeScript runtime or the Rust runtime.

This is the public ABI for `cli-tool` skills. It is not an embedded SDK
migration, provider integration, or broad TypeScript sunset task.

## Context

The existing skill examples use subprocess entrypoints:
- `oss/examples/hello-world/run.mjs`
- `oss/skills/scafld/run.mjs`
- `oss/skills/run.mjs`

Current TypeScript behavior:
- `oss/packages/adapters/src/cli-tool/index.ts` builds the author-visible
  `RUNX_INPUTS_JSON` / `RUNX_INPUTS_PATH` / `RUNX_INPUT_*` environment, drains
  stdout and stderr concurrently, and kills process groups on timeout.
- `oss/packages/runtime-local/src/runner-local/process-sandbox.ts` enforces cwd
  containment rules and always re-adds `RUNX_CWD` to the child environment.

Current Rust behavior:
- `oss/crates/runx-runtime/src/sandbox.rs` now re-adds `RUNX_CWD`, spills full
  input JSON above 48 KiB to `RUNX_INPUTS_PATH`, omits per-input env values
  above 8 KiB, matches TypeScript `RUNX_INPUT_*` normalization, and denies
  escaped cwd paths for non-`unrestricted-local-dev` policies. Workspace
  policy now uses `RUNX_CWD ?? INIT_CWD ?? current_dir`, matching the
  TypeScript sandbox boundary.
- `oss/crates/runx-runtime/src/adapters/cli_tool.rs` drains stdout/stderr while
  the child runs and kills the spawned process group on Unix timeouts, with a
  direct-child fallback on non-Unix.

## Objectives

- Freeze the author-facing subprocess ABI in docs and tests.
- Preserve `RUNX_CWD`, `RUNX_INPUTS_JSON`, `RUNX_INPUTS_PATH`, `RUNX_INPUT_*`,
  stdin mode, cwd policy, timeout, stdout/stderr, exit-code, and receipt
  behavior across TypeScript and Rust.
- Add executable conformance fixtures that run the same skill entrypoint through
  both runtimes while TypeScript still exists.
- Make large input and large output behavior safe and deterministic.
- Fail closed on unsupported author assumptions instead of silently drifting.

## Scope

In scope:
- Contract documentation for the subprocess skill ABI.
- Rust runtime parity fixes for author-visible ABI gaps if harden confirms this
  spec should own implementation.
- Cross-runtime fixture skills covering env, stdin, cwd, large inputs, large
  outputs, timeout tree cleanup, and stdout JSON parsing.
- Tests proving the contract is stable without relying on TypeScript-only
  helper packages.

Out of scope:
- Embedded SDK migration; owned by `embedded-sdk-migration-story`.
- Broad `@runxhq/runtime-local` deletion; owned by `rust-ts-sunset-runtime-local`.
- Provider-specific skill behavior.
- Rewriting skills that import `@runxhq/core`; this spec only defines the
  subprocess ABI those skills can choose to target.

## Dependencies

- `rust-runtime-skill-execution`
- `rust-cli-native-skill-run-foundation`
- `rust-ts-sunset-runtime-local`

## Touchpoints

- `oss/packages/adapters/src/cli-tool/index.ts`
- `oss/packages/runtime-local/src/runner-local/process-sandbox.ts`
- `oss/crates/runx-runtime/src/sandbox.rs`
- `oss/crates/runx-runtime/src/adapters/cli_tool.rs`
- `oss/crates/runx-runtime/tests/`
- `oss/fixtures/runtime/` or a new `oss/fixtures/contracts/skill-author-runtime/`

## Risks

- Treating this as a cleanup task will miss the real boundary: this is a public
  compatibility contract.
- Preserving unsafe ambient-env inheritance would weaken sandbox posture. The
  contract should preserve documented behavior, not accidental secret leakage.
- Cross-runtime tests can become brittle if they assert internal metadata rather
  than author-visible process behavior.

## Acceptance

Profile: strict

Definition of done:
- [x] `dod1` The skill author runtime contract v1 is documented in a stable
  location and names every guaranteed environment variable, stdin mode, output
  rule, cwd rule, timeout rule, and receipt expectation.
- [x] `dod2` The Rust runtime implements the same author-visible behavior as
  the TypeScript runtime for the v1 contract.
- [x] `dod3` A fixture `run.js`/`run.mjs` suite proves env, stdin, cwd, large
  input spill, large output drain, timeout cleanup, and stdout JSON behavior.
- [x] `dod4` The TypeScript and Rust runtimes run the same fixtures while both
  runtimes exist.
- [x] `dod5` Unsupported assumptions are documented as non-contractual and, if
  observable, fail closed.

Validation:
- [x] `v1` Rust cli-tool contract tests pass.
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test cli_tool_contract -- --nocapture`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-21 local command
    `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features cli-tool --test cli_tool_contract -- --nocapture`
    passed 13 tests including Unix descendant process cleanup.
- [x] `v2` TypeScript cli-tool contract tests pass while runtime-local exists.
  - Command: `pnpm vitest run packages/adapters/src/cli-tool/index.test.ts`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-21 local command passed 13 tests.
- [x] `v3` Cross-runtime fixture parity passes.
  - Command: `pnpm fixtures:skill-author-runtime:check`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-21 local command passed 6 TypeScript fixture cases and
    the Rust `skill_author_runtime_fixtures` test over the same fixture suite.
    Reconfirmed 2026-05-21T13:50:21Z: command passed 6 TypeScript fixture
    cases and 1 Rust fixture test.
- [x] `v4` Rust clippy passes for runtime changes.
  - Command: `cargo clippy --manifest-path crates/Cargo.toml -p runx-runtime --all-targets --features cli-tool -- -D warnings`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-21 local command passed after the cli-tool contract
    tests and payment-state compile repair.

## Phase 1: Contract Inventory

Status: done
Dependencies: none

Objective: Complete this phase.

Changes:
- Document the v1 subprocess ABI.
- Record non-contractual surfaces: receipt IDs, artifact IDs, `RUNX_HOME`, and package imports are not guaranteed unless explicitly documented.
- Record exact thresholds for `RUNX_INPUTS_JSON` and `RUNX_INPUTS_PATH`.

Acceptance:
- none

## Phase 2: Rust Parity

Goal: make Rust match the v1 author-facing contract.

Status: done
Dependencies: Phase 1

Changes:
- Re-add `RUNX_CWD` to the child environment. Done.
- Implement `RUNX_INPUTS_PATH` spill and per-input env-size caps. Done.
- Align input env-name normalization. Done.
- Drain stdout/stderr while the child is running. Done.
- Kill process groups or explicitly document and prove the platform-specific
  equivalent. Done on Unix; non-Unix retains direct-child timeout behavior.
- Enforce cwd containment denials matching the contract. Done.

Acceptance:
- Rust contract tests cover each repaired behavior landed so far, including
  Unix descendant cleanup on cli-tool timeout.

## Phase 3: Cross-Runtime Fixtures

Goal: prove the same skill entrypoints behave the same under both runtimes.

Status: done
Dependencies: Phase 2

Changes:
- Add fixture skills for env, stdin, cwd, large inputs, output pressure, timeout
  cleanup, and structured stdout.
- Add one command that runs the fixture suite through TypeScript and Rust and
  compares only author-visible behavior.

Acceptance:
- The fixture suite is the gate for future TypeScript sunset work.

## Rollback

Revert implementation changes while keeping the contract inventory if parity
implementation uncovers a design conflict. Do not complete TypeScript sunset
work until this spec or a successor records the accepted contract.

## Review

Review must verify the contract against both runtimes and reject any fix that
preserves accidental ambient secret exposure as a stable guarantee.

## Origin

User review of Rust migration risk on 2026-05-21 found that `run.js` is the
consumer ABI and that Rust currently drifts from the TypeScript subprocess
contract.
