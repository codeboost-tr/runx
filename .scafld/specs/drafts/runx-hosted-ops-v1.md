---
spec_version: '2.0'
task_id: runx-hosted-ops-v1
created: '2026-06-04T06:20:35Z'
updated: '2026-06-04T06:20:35Z'
status: draft
harden_status: not_run
size: medium
risk_level: medium
---

# runx-hosted-ops-v1

## Current State

Status: draft
Current phase: none
Next: approve
Reason: draft created
Blockers: none
Allowed follow-up command: `scafld approve runx-hosted-ops-v1`
Latest runner update: none
Review gate: not_started

Roadmap: Wave 4 (later). Sequenced AFTER the magnet + heroes; these gate a public
hosted launch, not the demo. Cloud is edit-restricted — this scopes work, the
cloud team executes.

## Summary

The hosted-product + ops initiatives for runx beyond the local CLI. The hosted
layer is further along than the docs imply but built on a spawn-per-run / serial-
drain topology, not the resident-kernel keystone the docs name. This scopes: the
resident-kernel transport (kernel run once behind the cloud-to-kernel bridge, with
per-principal isolation, not one binary per user — the unbuilt scaling keystone) +
process lifecycle at scale (general pooling / crash-recovery across many
integrations, beyond today's MCP session pooling); deploy ops (secrets, backups,
manifests, readiness — the payment-finality worker/config shape); marketplace trust
hardening (non-GitHub author verification, maturity-graduation automation, a
moderation/abuse surface); hosted product UX; and issuer-key publication.

## Objectives

- Resident-kernel transport: the kernel runs once behind the Phase-0 bridge with
  per-principal isolation (the scaling keystone), replacing spawn-per-run for
  hosted multi-tenant.
- Process lifecycle at scale: pooling, crash recovery, graceful degradation across
  dozens of concurrent integrations.
- Deploy ops: secrets, backups, deploy manifests, readiness gates that match the
  running processes (no parsed-but-unused flags).
- Marketplace trust hardening + hosted UX + issuer-key (JWKS) publication.

## Scope

In scope:
- The resident-kernel transport + process lifecycle keystone; deploy ops; the
  marketplace trust gaps (non-GitHub author verification, maturity-graduation
  automation, moderation/abuse); hosted UX polish.

Out of scope:
- The magnet + heroes (earlier waves); anything gating the demo (this gates the
  public hosted launch, not the launch demo).

## Dependencies

- SHIPPED: the cloud-to-kernel bridge (Phase 0); registry trust tiers + GitHub-app
  ownership claim + self-publish reindex.

## Assumptions

- The current spawn-per-run/serial-drain hosted topology works for low concurrency;
  the resident-kernel transport is a real rebuild for hosted scale, not a tweak.

## Touchpoints

- The cloud-to-kernel bridge; the hosted worker/runtime-service; deploy manifests +
  secrets/backups; the marketplace/registry trust surface; the issuer signing key.

## Risks

- **Topology mismatch.** Resident-kernel is a real rebuild vs spawn-per-run.
  Mitigation: stage it; keep spawn-per-run until the resident transport is proven.
- **Cloud edit-restriction.** This spec scopes; the cloud team executes; do not
  clobber cloud work.

## Acceptance

Profile: standard

Validation:
- A resident kernel serves multiple principals with isolation behind the bridge;
  process lifecycle handles N concurrent integrations with crash recovery; deploy
  ops (secrets/backups/manifests/readiness) are complete and match running
  processes; marketplace trust gaps closed; issuer pubkey published (JWKS).

## Phase 1: Resident-kernel transport + process lifecycle

Status: pending
Dependencies: cloud-to-kernel bridge (shipped)

Objective: hosted multi-tenant runs the kernel once, per-principal isolated, with
real process lifecycle.

Changes:
- Resident-kernel transport behind the bridge; pooling/crash-recovery/lifecycle.

Acceptance:
- [ ] `ac1` manual - resident kernel serves multiple principals with isolation + recovery
  - Expected kind: `manual`
  - Status: pending

## Phase 2: Deploy ops + marketplace trust + hosted UX

Status: pending
Dependencies: Phase 1

Objective: a public hosted launch is operationally ready.

Changes:
- Secrets/backups/manifests/readiness; non-GitHub author verification + maturity
  automation + moderation; issuer pubkey (JWKS) publication; hosted UX.

Acceptance:
- [ ] `ac2` manual - deploy ops + marketplace trust + key publication complete
  - Expected kind: `manual`
  - Status: pending

## Rollback

- Staged; keep spawn-per-run until the resident transport is proven. Cloud-side;
  no OSS kernel change required.

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
