---
spec_version: '2.0'
task_id: runx-thread-outbox-provider-front-v1
created: '2026-06-04T06:20:35Z'
updated: '2026-06-04T06:20:35Z'
status: draft
harden_status: not_run
size: medium
risk_level: medium
---

# runx-thread-outbox-provider-front-v1

## Current State

Status: draft
Current phase: none
Next: approve
Reason: draft created
Blockers: none
Allowed follow-up command: `scafld approve runx-thread-outbox-provider-front-v1`
Latest runner update: none
Review gate: not_started

Roadmap: Wave 4 (later). Migrates the provider-mutation boundary into the kernel;
the live TS path ships today, so this is deferred until worth moving.

## Summary

Migrate the `issue-to-pr` provider-mutation boundary from the live TS
`thread.push_outbox` path into the kernel's Rust thread-outbox-provider front, and
land governed-execution-layer item 15 (the post-merge publisher on that front). A
complete Rust supervisor already exists
(`oss/crates/runx-runtime/src/outbox_provider.rs`:
`ThreadOutboxProviderProcessSupervisor` with `invoke_push`/`invoke_fetch`) plus the
`thread-outbox-provider-protocol-v1` contract + JSON schemas, but it is NOT
dispatched as a SourceKind or graph step (inert, unit-tested only). This wires it
into dispatch so provider mutation seals through the governed Rust front, with
parity before any cutover of the TS path.

## Objectives

- Dispatch the thread-outbox-provider front so `issue-to-pr`'s push step routes
  through the governed Rust front, sealing a receipt for the provider mutation.
- The item-15 post-merge publisher on the same front.
- Provider tokens delivered via Rust-supervised `CredentialDelivery`.

## Scope

In scope:
- Wire the inert `ThreadOutboxProviderProcessSupervisor` into dispatch; route
  `issue-to-pr` push through it; the post-merge publisher; harness + parity.

Out of scope:
- Removing the TS `thread.push_outbox` path before Rust-front parity is proven.
- New providers beyond the GitHub thread/outbox lane.

## Dependencies

- The built-but-inert Rust supervisor; the `thread-outbox-provider-protocol-v1`
  contract + schemas (kept); credential delivery; the TS `issue-to-pr` graph.

## Assumptions

- The protocol + supervisor are the right contract; this is a dispatch-wiring +
  migration, not a rebuild.

## Touchpoints

- `oss/crates/runx-runtime/src/outbox_provider.rs`; the SourceKind/graph-step
  dispatch; `skills/issue-to-pr/X.yaml` (the push step); the post-merge publisher.

## Risks

- **Skill-safety (highest).** `issue-to-pr` must keep working through the
  migration. Mitigation: prove Rust-front parity with the TS path first; do not cut
  the TS path until parity holds; keep the contract surface frozen.

## Acceptance

Profile: strict

Validation:
- `issue-to-pr`'s push seals through the Rust thread-outbox-provider front; the
  post-merge publisher seals a receipt; `issue-to-pr` still runs green end to end;
  no SourceKind/protocol surface removed; gates green.

## Phase 1: Dispatch the front + route issue-to-pr push (parity)

Status: pending
Dependencies: the inert supervisor, the protocol contract

Objective: provider mutation seals through the governed Rust front at parity with
the TS path.

Changes:
- Wire the supervisor into dispatch; route the `issue-to-pr` push step; harness +
  parity assertions.

Acceptance:
- [ ] `ac1` command - issue-to-pr push seals via the Rust front, graph still green
  - Command: `runx harness skills/issue-to-pr/<push-case>.yaml --json`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Phase 2: Post-merge publisher (item 15)

Status: pending
Dependencies: Phase 1

Objective: a merged change publishes through the governed front, sealing a receipt.

Changes:
- Implement the post-merge publisher on the thread-outbox-provider front.

Acceptance:
- [ ] `ac2` command - post-merge publish seals
  - Command: `runx harness examples/post-merge-publish/<case>.yaml --json`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Rollback

- Additive dispatch; revert routing to the TS `thread.push_outbox` path (kept until
  parity). No contract churn.

## Review

Status: not_started
Verdict: none

Findings:
- none

## Self Eval

- none

## Deviations

- none

## Metadata

- created_by: scafld

## Origin

Created by: scafld
Source: plan

## Harden Rounds

- none

## Planning Log

- none
