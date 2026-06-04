---
spec_version: '2.0'
task_id: runx-github-mcp-hero-v1
created: '2026-06-04T06:20:35Z'
updated: '2026-06-04T06:36:00Z'
status: active
harden_status: not_run
size: medium
risk_level: medium
---

# runx-github-mcp-hero-v1

## Current State

Status: active
Current phase: phase1
Next: build
Reason: phase phase1 opened
Blockers: none
Allowed follow-up command: `scafld handoff runx-github-mcp-hero-v1`
Latest runner update: 2026-06-04T06:36:00Z
Review gate: not_started

## Summary

Ship the first hero (executable today, gates nothing): governed GitHub via the MCP
adapter, scope-bounded, receipt-sealed. Grant `repo.read`, the agent reads an
issue/PR fine, attempts an out-of-scope write, admission REFUSES, and the sealed
denial receipt verifies offline. Plus the two read-only siblings that make it read
as a category: governed issue-triage (emit a triage packet + receipt) and a
governed PR-review note (one scoped comment via the public-comment path). The
survey found NO `type: mcp` GitHub example anywhere under `oss/examples` today, so
this builds the first one — the demo north-star.md names as the executable hero.

## Objectives

- A GitHub MCP-source skill that reads under `repo.read`, with a sealed receipt and
  an offline-verifiable governed REFUSAL of an out-of-scope write — the headline demo.
- Wire the existing `issue-triage` skill (already references a real github issue_url
  + comment channel) to the MCP read path; it emits a triage packet + receipt with
  no mutation scope exercised.
- A new governed PR-review-note sibling: one scoped comment via the public-comment
  path, refusing any push/merge attempt.
- Each reaches a maturity tier with a harness case; `receipt-auditor` +
  `least-privilege-auditor` confirm no out-of-scope scope was exercised.

## Scope

In scope:
- The GitHub MCP-source example skill + the refusal demo (grant repo.read → attempt
  write → sealed denial → offline verify with `verify.mjs`).
- The `issue-triage` sibling wired to MCP read; the new PR-review-note sibling.
- Harness cases + maturity tiering for all three (these are first-party `@runx`
  anchors that may reach stable per the maturity-tiers rule).

Out of scope:
- Non-GitHub providers (BYO connect-session, separate).
- Any write beyond the single scoped PR comment; auto-merge; PR-opening (that is the
  TS `issue-to-pr` lane, not this hero).

## Dependencies

- SHIPPED: the MCP adapter (`serve_mcp_json_rpc`) + the MCP-source client, agent-step,
  GitHub connect (the only wired provider), receipts, authority/scopes, the harness.
- The governed-tool-call convention for the scoped-comment mutation (PR-review note).

## Assumptions

- GitHub is the only wired provider (north-star.md); the hero is scoped to it.
- The refusal must be a REAL admission denial (over-scope attempt), not a staged one,
  so the demo's credibility holds.

## Touchpoints

- A new `oss/examples/github-mcp-*` (the first MCP-source example) + harness.
- `oss/skills/issue-triage` (wire to MCP read); a new PR-review-note skill.
- The MCP adapter + GitHub connect path; `receipt-auditor`, `least-privilege-auditor`.
- `verify.mjs` (offline-verify the denial + the triage receipts).

## Risks

- **Mutation scope creep.** The PR-review note must be gated to exactly the comment
  scope; an over-broad scope undermines the least-privilege story. Mitigation: bound
  the scope + assert refusal of push/merge in the harness.
- **Demo credibility.** A staged refusal reads as theatre. Mitigation: the refusal is
  a genuine admission denial of an out-of-scope act, sealed + offline-verifiable.

## Acceptance

Profile: strict

Validation:
- The GitHub MCP read hero: a scoped read seals a receipt; an out-of-scope write is
  refused at admission with a sealed denial that `verify.mjs` confirms offline.
- The two siblings run: `issue-triage` emits a packet + receipt (no mutation scope
  exercised, confirmed by `least-privilege-auditor`); the PR-review note posts one
  scoped comment and refuses push/merge.
- `pnpm fixtures:harness:check` + the new harness cases pass; the three skills are
  maturity-tiered and locked.

## Phase 1: GitHub MCP read hero + refusal demo

Status: active
Dependencies: MCP adapter, GitHub connect (shipped)

Objective: the first MCP-source GitHub example with the grant→read→refuse→sealed-

Changes:
- Build the GitHub MCP-source example skill + harness; the refusal demo + run script.

Acceptance:
- [ ] `ac1` command - read seals, out-of-scope write refused + verifies offline
  - Command: `runx harness examples/github-mcp-hero/<case>.yaml --json && node examples/governed-spend/verify.mjs <denial-receipt>`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Phase 2: The two read-only siblings (issue-triage + PR-review note)

Status: pending
Dependencies: Phase 1

Objective: the category-defining siblings, each sealed + scope-audited.

Changes:
- Wire `issue-triage` to MCP read; build the scoped PR-review-note skill; harness +
  maturity for both.

Acceptance:
- [ ] `ac2` command - siblings seal + least-privilege holds
  - Command: `runx harness skills/issue-triage/<case>.yaml --json && runx harness skills/pr-review-note/<case>.yaml --json`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Rollback

- Additive examples/skills. Remove the github-mcp example + the PR-review-note skill
  and revert the issue-triage wiring; no contract or SourceKind change.

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
