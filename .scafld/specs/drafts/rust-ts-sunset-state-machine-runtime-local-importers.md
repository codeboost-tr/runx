---
spec_version: '2.0'
task_id: rust-ts-sunset-state-machine-runtime-local-importers
created: '2026-05-22T00:58:00+10:00'
updated: '2026-05-22T00:58:00+10:00'
status: draft
harden_status: not_run
size: small
risk_level: high
---

# State-machine sunset: runtime-local importers

## Current State

Status: draft
Current phase: planning only
Next: classify each runtime-local transition/planning importer and choose
whether it can move to the existing kernel bridge, a new Rust runtime boundary,
or a later runtime-local retirement slice.
Reason: `rust-ts-sunset-state-machine` is blocked by live runtime-local
state-machine consumers. The completed prerequisite slice moved only
sequential graph state creation for `prepare-run.ts`; the remaining consumers
are synchronous transition, planning, fanout, hydration, governance, and test
paths that cannot be removed by deleting the TS state-machine package.
Blockers: runtime ownership for transition/planning semantics is not settled,
fanout gate/governance shape ownership is still coupled to TS types, and the
fixture generators still use the TS implementation as their oracle.
Allowed follow-up command: `scafld validate rust-ts-sunset-state-machine-runtime-local-importers --json`
Latest runner update: 2026-05-22T00:58:00+10:00 - child draft created from the
fresh importer census. No production imports moved in this slice.
Review gate: not_started

## Summary

Plan the runtime-local importer migration needed before the parent
`rust-ts-sunset-state-machine` deletion draft can advance. This is not a
deletion spec. It must not remove `packages/core/src/state-machine/**`, must
not remove `packages/core/package.json` `exports["./state-machine"]`, and must
not add a compatibility shim for `@runxhq/core/state-machine`.

The safe migration shape is expected to be incremental:
- keep the existing kernel bridge for any operation that can tolerate async
  `runx kernel eval`;
- create an explicit Rust runtime-owned boundary for transition/planning
  operations that cannot be safely handled by one-off kernel calls;
- leave fixture-oracle ownership to the parent or a separate fixture-freeze
  decision.

## Objectives

- Classify each runtime-local `@runxhq/core/state-machine` importer by
  behavior: planning, transition, fanout decision, fanout key, hydration,
  governance type surface, and single-step state carrier.
- Identify the smallest safe runtime-local importer migrations that have
  obvious targeted tests.
- Keep deletion blocked until a fresh parent census proves all live imports are
  gone.
- Avoid production import churn unless the replacement boundary is explicit and
  covered by focused tests.

## Scope

In scope:
- `packages/runtime-local/src/runner-local/orchestrator.ts`
- `packages/runtime-local/src/runner-local/index.ts`
- `packages/runtime-local/src/runner-local/graph-hydration.ts`
- `packages/runtime-local/src/runner-local/graph-fanout-gates.ts`
- `packages/runtime-local/src/runner-local/graph-governance.ts`
- `packages/runtime-local/src/runner-local/orchestrator/hydrate-resume.ts`
- `packages/runtime-local/src/runner-local/orchestrator/handle-run-step.ts`
- `packages/runtime-local/src/runner-local/orchestrator/handle-run-fanout.ts`
- `packages/runtime-local/src/runner-local/orchestrator/handle-terminal.ts`
- `packages/runtime-local/src/runner-local/orchestrator/handle-paused.ts`
- `tests/graph-hydration-orphan-start.test.ts`, only as validation or follow-up
  cleanup for migrated runtime-local behavior.

Out of scope:
- Deleting `packages/core/src/state-machine/**`.
- Removing `packages/core/package.json` `exports["./state-machine"]`.
- Changing fixture generators or fixture ownership.
- Payments, MCP, target-runner, post-merge observer, embedded-sdk,
  TS-boundary, parser/runtime-local, external-adapter, and rust-dev work.

## Dependencies

- `rust-ts-sunset-state-machine` remains the deletion parent and stays blocked.
- A Rust runtime ownership decision for synchronous graph transition/planning.
- A separate fixture-generator ownership or freeze decision before final TS
  state-machine deletion.

## Importer Census

Checked on 2026-05-22:

```bash
rg -n "from ['\"]@runxhq/core/state-machine['\"]|from ['\"].*state-machine/index\.js['\"]" packages/runtime-local/src tests scripts --glob '!**/dist/**' --glob '!node_modules' --glob '!target'
rg -l "from ['\"]@runxhq/core/state-machine['\"]" packages/runtime-local/src --glob '!**/dist/**' | sort
```

Observed results:
- 13 live importer files across runtime-local, tests, and scripts.
- 10 runtime-local source files import `@runxhq/core/state-machine`.
- One root graph hydration test imports `@runxhq/core/state-machine`.
- Two fixture generator scripts import `packages/core/src/state-machine/index.js`
  directly.

Runtime-local importers:
- `packages/runtime-local/src/runner-local/orchestrator.ts`
- `packages/runtime-local/src/runner-local/index.ts`
- `packages/runtime-local/src/runner-local/graph-hydration.ts`
- `packages/runtime-local/src/runner-local/graph-fanout-gates.ts`
- `packages/runtime-local/src/runner-local/graph-governance.ts`
- `packages/runtime-local/src/runner-local/orchestrator/hydrate-resume.ts`
- `packages/runtime-local/src/runner-local/orchestrator/handle-run-step.ts`
- `packages/runtime-local/src/runner-local/orchestrator/handle-run-fanout.ts`
- `packages/runtime-local/src/runner-local/orchestrator/handle-terminal.ts`
- `packages/runtime-local/src/runner-local/orchestrator/handle-paused.ts`

## Acceptance

Profile: standard

Definition of done:
- [ ] `dod1` Runtime-local state-machine importers are classified with explicit
  owner, replacement boundary, and test target.
- [ ] `dod2` No TS state-machine implementation files are deleted or renamed.
- [ ] `dod3` No compatibility shim, re-export, fallback adapter, or legacy
  `@runxhq/core/state-machine` shape is added.
- [ ] `dod4` Parent deletion draft remains blocked until runtime-local and
  fixture-oracle imports are cleared.

Validation:
- [ ] `v1` Scafld validates this spec.
  - Command: `scafld validate rust-ts-sunset-state-machine-runtime-local-importers --json`
  - Expected kind: `exit_code_zero`
  - Status: pending
  - Evidence: none
- [ ] `v2` Runtime-local importer census is current.
  - Command: `rg -n "from ['\"]@runxhq/core/state-machine['\"]" packages/runtime-local/src --glob '!**/dist/**'`
  - Expected kind: `exit_code_zero`
  - Status: pending
  - Evidence: none
- [ ] `v3` TS state-machine implementation remains present.
  - Command: `test -d packages/core/src/state-machine`
  - Expected kind: `exit_code_zero`
  - Status: pending
  - Evidence: none

## Phase 1: Importer Classification

Status: pending
Dependencies: none

Goal: map every runtime-local state-machine import to an ownership decision and
replacement strategy.

Acceptance:
- [ ] `ac1` command - Runtime-local state-machine importer census is current.
  - Command: `rg -n "from ['\"]@runxhq/core/state-machine['\"]" packages/runtime-local/src --glob '!**/dist/**'`
  - Expected kind: `exit_code_zero`
  - Status: pending
  - Evidence: none
- [ ] `ac2` command - Importer assignments are recorded in this spec.
  - Command: `rg -n "Runtime-local importers:" .scafld/specs/drafts/rust-ts-sunset-state-machine-runtime-local-importers.md`
  - Expected kind: `exit_code_zero`
  - Status: pending
  - Evidence: none

## Phase 2: Safe Migration

Status: pending
Dependencies: Phase 1

Goal: move only importer classes with an explicit Rust-owned boundary and
focused tests.

Acceptance:
- [ ] `ac3` command - Migrated runtime-local imports no longer reference the
  TS public export.
  - Command: `rg -n "from ['\"]@runxhq/core/state-machine['\"]" packages/runtime-local/src --glob '!**/dist/**'`
  - Expected kind: `no_matches`
  - Status: pending
  - Evidence: none
- [ ] `ac4` command - TS state-machine implementation is untouched.
  - Command: `test -d packages/core/src/state-machine`
  - Expected kind: `exit_code_zero`
  - Status: pending
  - Evidence: none

## Rollback

- Revert only this planning or importer migration slice. Do not restore, remove,
  or replace TS state-machine implementation files from this spec.

## Metadata

- created_by: codex
