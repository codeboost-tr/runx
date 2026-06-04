---
spec_version: '2.0'
task_id: runx-byo-connect-portfolio-v1
created: '2026-06-04T06:20:35Z'
updated: '2026-06-04T06:20:35Z'
status: draft
harden_status: not_run
size: medium
risk_level: medium
---

# runx-byo-connect-portfolio-v1

## Current State

Status: draft
Current phase: none
Next: approve
Reason: draft created
Blockers: none
Allowed follow-up command: `scafld approve runx-byo-connect-portfolio-v1`
Latest runner update: none
Review gate: not_started

Roadmap: Wave 2 (the platform unlock) feeding Wave 3 (the non-GitHub portfolio).
The highest-leverage platform work after the magnet + heroes.

## Summary

Close the one remaining PLATFORM gap — the BYO connect-session for non-GitHub
providers — and build out the demand-shaped non-GitHub skill portfolio it unlocks.
The cloud side is shipped/reviewed-pass (byo-credential-foundations: encrypted BYO
credential store, scoped grants); the missing piece is per-provider OAuth
registration + the CLI connect-session UX. Once delivered, the already-shipped HTTP
front reaches the ~351 byo-ready REST providers with NO per-provider runtime code,
unlocking the skill-seeds portfolio (search / mail / calendar / db / browser).

## Objectives

- Per-provider OAuth registration + a CLI connect-session UX that derives a scoped
  grant for a non-GitHub provider.
- A non-GitHub provider connected + a governed skill running a read under that grant
  with a sealed receipt and an out-of-scope refusal.
- The first portfolio skills over the http/external-adapter fronts (sql-analyst,
  inbox-and-calendar-exec, knowledge-router, deep-research-brief, lead-enrichment),
  each maturity-tiered with a harness case.

## Scope

In scope:
- The connect-session UX + per-provider OAuth registration (the OSS/CLI side of the
  reviewed cloud store).
- The first ~5 non-GitHub skills over the shipped http front (and the OpenAPI front
  for spec-backed APIs); harness + maturity tiering.

Out of scope:
- GitHub (already wired).
- Building portfolio skills before the connect-session lands.
- Deep per-provider polish / the full ~351-provider sprawl (start with high-demand).

## Dependencies

- SHIPPED: the BYO credential store + scoped grants (cloud, reviewed-pass); the HTTP
  front; credential delivery.
- The OpenAPI front (Wave 2) for spec-backed providers.

## Assumptions

- The HTTP front already governs any REST provider once a credential is delivered
  (verified shipped: method+URL+headers, SSRF/private-net opt-in, `${secret:NAME}`
  headers), so the portfolio is gated only on the connect-session, not runtime code.

## Touchpoints

- The cloud BYO store + connect endpoints; a new CLI connect-session command; the
  HTTP front (`adapters/http.rs`); the new portfolio skills + the official lock +
  maturity tiers.

## Risks

- **Provider sprawl.** Mitigation: start with a few high-demand providers; the front
  generalizes, the demand does not.
- **Auth-scope correctness.** Mitigation: scoped grants + least-privilege, asserted
  by `least-privilege-auditor` in the harness.

## Acceptance

Profile: strict

Validation:
- A non-GitHub provider connects via the session and derives a scoped grant; a
  governed read seals a receipt; an out-of-scope call is refused.
- The first portfolio skills run, seal receipts, and are maturity-tiered + locked.
- `pnpm verify:fast` + the new harness cases green.

## Phase 1: Connect-session UX + per-provider OAuth + one provider

Status: pending
Dependencies: BYO cloud store (shipped), HTTP front (shipped)

Objective: a non-GitHub provider is connectable end to end with a scoped grant.

Changes:
- Per-provider OAuth registration; the CLI connect-session UX; one provider wired +
  a governed read skill.

Acceptance:
- [ ] `ac1` command - non-GitHub provider connects + governed read seals
  - Command: `runx harness skills/<byo-read-skill>/<case>.yaml --json`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Phase 2: The first non-GitHub portfolio skills

Status: pending
Dependencies: Phase 1

Objective: the demand-shaped seeds run over the http/external-adapter fronts.

Changes:
- Build sql-analyst, inbox-and-calendar-exec, knowledge-router, deep-research-brief,
  lead-enrichment; harness + maturity.

Acceptance:
- [ ] `ac2` command - portfolio skills run + are tiered
  - Command: `runx harness skills/<each-seed>/<case>.yaml --json`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Rollback

- The connect-session is additive platform UX; portfolio skills are additive +
  maturity-gated (alpha first). Remove the skills + the connect command; no kernel
  or contract change.

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
