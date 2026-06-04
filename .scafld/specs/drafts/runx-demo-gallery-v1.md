---
spec_version: '2.0'
task_id: runx-demo-gallery-v1
created: '2026-06-04T06:20:35Z'
updated: '2026-06-04T06:20:35Z'
status: draft
harden_status: not_run
size: medium
risk_level: medium
---

# runx-demo-gallery-v1

## Current State

Status: draft
Current phase: none
Next: approve
Reason: draft created
Blockers: none
Allowed follow-up command: `scafld approve runx-demo-gallery-v1`
Latest runner update: none
Review gate: not_started

Roadmap: cross-cutting (grows with each wave). The offline-verifier promotion is
launch-blocking for the real-rail demo.

## Summary

A demos/showcase surface that links each runnable example to its one-line proof and
its sealed, offline-verifiable receipt, turning the shipped + new heroes into a
coherent product story: the governed GitHub refusal, offline verify, the payment
launch demo, OpenAPI multi-spec, and the per-lane fronts. Distinct from frantic.md
(the separate capture-venue brand); this is a runx `/demos` surface (and/or a
recorded asciinema set). It also promotes the offline verifier
(`examples/governed-spend/verify.mjs`) into a standalone reusable verifier with a
JWKS-style issuer-pubkey discovery endpoint — the objection-killer carried across
every hero and launch-blocking for the real-rail demo.

## Objectives

- A curated demo gallery: each demo = a runnable example + a one-line proof + a
  sealed receipt the viewer verifies offline.
- Feature the GitHub hero trio, governed-spend, the offline-verify demo, OpenAPI,
  and the per-lane fronts; add the payment HN artifact when it lands.
- Promote `verify.mjs` to a standalone reusable verifier + publish the issuer pubkey
  via a JWKS-style discovery endpoint.

## Scope

In scope:
- The `/demos` gallery surface; wiring each runnable example to its proof + receipt;
  the standalone verifier + pubkey discovery; a harness case per featured demo so a
  demo cannot silently rot.

Out of scope:
- frantic.md (separate brand/venue); marketing polish; demos for unbuilt lanes
  (link them as they land).

## Dependencies

- The heroes/examples (GitHub hero — Wave 0; payment — Wave 1; OpenAPI — Wave 2);
  `verify.mjs` (shipped, to promote); the issuer signing key (for pubkey discovery).

## Assumptions

- Each featured demo already (or will) ship as a runnable example with a sealed
  receipt; the gallery curates + links, it does not re-implement.

## Touchpoints

- A site `/demos` surface (or asciinema set); `oss/examples/**/run.sh`; the promoted
  standalone verifier; the JWKS-style pubkey endpoint; harness cases per demo.

## Risks

- **Demo rot.** A linked demo that silently breaks is worse than none. Mitigation:
  every gallery demo has a harness case gated in CI.

## Acceptance

Profile: standard

Validation:
- The gallery lists each runnable example with its one-line proof and a receipt the
  viewer verifies offline with the standalone verifier + pubkey discovery.
- Each featured demo has a CI-gated harness case.

## Phase 1: Gallery surface + the shipped demos + verifier promotion

Status: pending
Dependencies: the shipped examples, verify.mjs

Objective: a curated gallery of the demos that ship today, each offline-verifiable.

Changes:
- Build the `/demos` surface; wire the shipped examples + proofs + receipts; promote
  the standalone verifier + JWKS pubkey discovery; harness case per demo.

Acceptance:
- [ ] `ac1` command - the standalone verifier confirms a featured demo receipt offline
  - Command: `node tools/verify/verify.mjs <featured-receipt> --jwks <pubkey-discovery-url>`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Phase 2: Add the heroes as they land (payment, OpenAPI, per-lane)

Status: pending
Dependencies: the respective wave demos

Objective: the gallery grows with each wave.

Changes:
- Add the payment HN artifact, the OpenAPI multi-spec demo, and per-lane demos as
  they ship.

Acceptance:
- [ ] `ac2` command - each newly-featured demo has a gated harness case
  - Command: `pnpm fixtures:harness:check`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Rollback

- Additive surface; remove the gallery + the standalone verifier export (the
  in-example verify.mjs remains).

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
