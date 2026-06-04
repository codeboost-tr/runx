---
spec_version: '2.0'
task_id: runx-receipt-content-addressing-v1
created: '2026-06-04T06:20:35Z'
updated: '2026-06-04T06:20:35Z'
status: draft
harden_status: not_run
size: medium
risk_level: medium
---

# runx-receipt-content-addressing-v1

## Current State

Status: draft
Current phase: none
Next: approve
Reason: draft created
Blockers: none
Allowed follow-up command: `scafld approve runx-receipt-content-addressing-v1`
Latest runner update: none
Review gate: not_started

Roadmap: Wave 2 (opportunistic, near-zero cost). The full claim-graph S-tier and
the "enlightened" receipt tier are EXPLICIT DO-NOT-BUILD forbidders — this spec is
ONLY the two cheap, compounding moves.

## Summary

Take the two cheap content-addressing moves on receipts now, while they are
near-free, to keep the claim-graph door open without retrofitting later: (1)
content-addressed receipt ids — two runs that did the identical governed thing
produce the same receipt id (free dedup); (2) a shared receipt/resolution envelope
so a verifier can walk a receipt's ancestry offline. Receipts already seal
(Ed25519, verifiable via `governed-spend/verify.mjs`). This spec is deliberately
bounded: it does NOT build the full receipt claim-graph S-tier or the enlightened
tier (both are reserved-draft DO-NOT-BUILD).

## Objectives

- A content-addressed receipt id (deterministic function of the canonical body),
  enabling free dedup of identical governed actions.
- A shared receipt/resolution envelope that lets a verifier walk ancestry offline.
- Both ADDITIVE: no canonical-JSON churn or digest change for existing receipts.

## Scope

In scope:
- The two moves, additive, with fixtures proving existing receipt digests are
  unchanged.

Out of scope:
- The full receipt claim-graph S-tier (reserved draft, DO-NOT-BUILD here).
- The "enlightened" receipt tier (reserved draft, DO-NOT-BUILD).
- Any non-additive change to `runx.receipt.v1` / canonical JSON.

## Dependencies

- The receipt contract (`runx.receipt.v1`) + canonical-JSON oracle; `verify.mjs`.

## Assumptions

- The two moves are additive over the sealed receipt and compounding (cheap now,
  expensive to retrofit), per the reserved-draft analysis.

## Touchpoints

- `crates/runx-contracts/src/receipt.rs` + the canonical-JSON oracle/fixtures;
  `examples/governed-spend/verify.mjs` (ancestry walk).

## Risks

- **Canonical-JSON churn.** A non-additive id/envelope change re-hashes every
  existing receipt. Mitigation: additive-only; gate on unchanged existing digests.
- **Scope creep into the forbidden full claim-graph.** Mitigation: hard-stop at the
  two moves; the S-tier/enlightened tiers stay DO-NOT-BUILD.

## Acceptance

Profile: strict

Validation:
- Two identical governed runs produce the same receipt id (dedup); a verifier walks
  a receipt's ancestry offline.
- Existing receipt digests/canonical oracle are UNCHANGED (`pnpm fixtures:harness:check`,
  the c14n oracle, `cargo nextest run --workspace --all-features` all green).

## Phase 1: Content-addressed receipt id (additive)

Status: pending
Dependencies: receipt contract + c14n oracle

Objective: identical governed actions yield the same receipt id; existing digests
unchanged.

Changes:
- Add the content-addressed id (additive field/derivation); fixtures proving dedup +
  unchanged existing digests.

Acceptance:
- [ ] `ac1` command - dedup holds, existing oracle unchanged
  - Command: `cargo nextest run --manifest-path crates/Cargo.toml -p runx-receipts && pnpm fixtures:harness:check`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Phase 2: Shared receipt/resolution envelope (offline ancestry)

Status: pending
Dependencies: Phase 1

Objective: a verifier walks a receipt's ancestry offline.

Changes:
- Add the shared resolution envelope (additive); extend `verify.mjs` to walk ancestry.

Acceptance:
- [ ] `ac2` command - offline ancestry walk verifies
  - Command: `node examples/governed-spend/verify.mjs <receipt> --walk-ancestry`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Rollback

- Additive fields; remove them. If any existing digest churns, the change was not
  additive — revert.

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
