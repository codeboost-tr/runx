---
spec_version: '2.0'
task_id: runx-process-helper-feature-boundary-v1
created: '2026-05-27T00:00:00Z'
updated: '2026-05-26T23:01:50Z'
status: completed
harden_status: not_run
size: small
risk_level: medium
---

# runx process helper feature boundary v1

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-26T23:01:50Z
Review gate: pass

## Summary

Make the shared runtime process helper available to all runtime modules that
compile by default. This preserves the single process-group signaling owner
introduced by `runx-process-supervisor-unification-v1` without restoring local
duplicate kill/configuration code in `outbox_provider`.

## Scope

- `crates/runx-runtime/src/lib.rs`
- `crates/runx-runtime/src/process_signal.rs`
- `crates/runx-runtime/src/process.rs`
- `crates/runx-runtime/src/process/signal.rs`
- `crates/runx-runtime/src/outbox_provider.rs`
- `crates/runx-runtime/src/adapters/mcp/transport.rs`

Out of scope:

- Runtime execution/S-tier files currently dirty in another lane.
- New process behavior.
- Reintroducing `/bin/kill` shell-outs or duplicate process-group helpers.

## Objectives

- Fix default-feature compile for runtime code that imports the shared process
  helper.
- Keep the no-duplicate-supervisor boundary intact.

## Acceptance

- `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test dev -- --nocapture`
- `cargo check --manifest-path crates/Cargo.toml -p runx-runtime`
- `! rg -n 'Command::new\\("/bin/kill"\\)' crates/runx-runtime/src --glob '*.rs'`
- `rustfmt --check crates/runx-runtime/src/lib.rs crates/runx-runtime/src/process_signal.rs crates/runx-runtime/src/process.rs crates/runx-runtime/src/process/signal.rs crates/runx-runtime/src/outbox_provider.rs crates/runx-runtime/src/adapters/mcp/transport.rs`

## Phase 1: Boundary Fix

Status: completed
Dependencies: none

Objective: Complete this phase.

Changes:
- Move process-group signaling into a small always-available internal module.
- Keep the full process supervisor module feature-gated.

Acceptance:
- none

## Phase 2: Focused Validation

Status: completed
Dependencies: phase1

Objective: Complete this phase.

Changes:
- Re-run the default-feature runtime compile/test that exposed the regression.

Acceptance:
- none

## Review

Status: completed
Verdict: pass
Mode: verify
Summary: Human-reviewed override accepted: Reviewed targeted feature-boundary fix after default runtime cargo check, default runtime dev test, mcp-feature runtime cargo check, no /bin/kill guard, and rustfmt --check on touched files passed. The shared process-group primitive is split from the feature-gated full supervisor, avoiding duplicate kill logic and default-feature warning churn.

Attack log:
- `review gate`: manual human audit -> clean (Reviewed targeted feature-boundary fix after default runtime cargo check, default runtime dev test, mcp-feature runtime cargo check, no /bin/kill guard, and rustfmt --check on touched files passed. The shared process-group primitive is split from the feature-gated full supervisor, avoiding duplicate kill logic and default-feature warning churn.)

Findings:
- none

