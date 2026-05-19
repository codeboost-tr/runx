---
spec_version: '2.0'
task_id: runx-operational-policy-config
created: '2026-05-19T02:08:02Z'
updated: '2026-05-19T02:30:00Z'
status: draft
harden_status: not_run
size: medium
risk_level: high
---

# Runx operational policy config

## Current State

Status: draft
Current phase: none
Next: harden
Reason: production issue-to-PR flows currently depend on repo-local policy files,
allowlists, runner availability assumptions, Slack routing, and outcome behavior
that should be visible, validated, and reusable through runx/Aster core.
Blockers: none; this should land before target-repo runners and outcome observer
become broadly available.
Allowed follow-up command: `scafld harden runx-operational-policy-config`
Latest runner update: none
Review gate: not_started

## Summary

Create a typed runx operational policy contract for repository routing,
ownership assignment, runner availability, Slack/source-thread routing, outcome
handling, dedupe behavior, and allowed automation actions. The policy must be
machine-validated, safe to display in Aster/admin surfaces, and explicit enough
that adopter repos like Nitrosend only supply thin product-specific mappings.

## Context

CWD: `.` (runx OSS workspace)

Current production learnings from Nitrosend:
- Issue intake can originate in Slack, Sentry, GitHub comments, or manual
  commands.
- The source issue/thread can differ from the target repo receiving the PR.
- Owner routing needs to be explicit and reviewable.
- Slack follow-ups must return to the original source thread, not channel root.
- Human review/merge gates must stay explicit for mutating code changes.
- Dedupe and outcome behavior need policy, not ad hoc script branches.

Candidate core surfaces:
- `skills/issue-intake/**`
- `skills/issue-to-pr/**`
- `skills/work-plan/**`
- `packages/cli/**`
- `crates/runx-runtime/**`
- Aster policy/admin read models

Invariants:
- Policy is not a secret store. Tokens, credentials, and private keys stay in
  runtime secrets.
- Unknown target repos, unknown runners, or unknown Slack routes fail closed.
- Policy distinguishes review-only automation, PR-producing automation, and any
  future merge-capable automation.
- Repo-local wrappers may narrow policy but should not reimplement core routing.

## Objectives

- Define a versioned policy schema for routing, owners, runners, Slack/source
  threads, dedupe, and outcomes.
- Add validation errors that are actionable for operators.
- Provide a readback/projection suitable for Aster/admin surfaces.
- Add fixture policies for Nitrosend-like multi-repo routing and minimal
  single-repo routing.
- Document where adopter-specific config ends and runx core behavior begins.

## Scope

In scope:
- Policy schema and parser.
- Validation for target repos, runner names, owner mappings, channel/thread
  routes, outcome settings, dedupe keys, and automation permissions.
- CLI/runtime helpers that consume the policy without duplicating parsing.
- Aster-facing safe projection shape.
- Tests and fixtures.

Out of scope:
- Secrets management implementation.
- Actual target-repo runner execution; owned by `runx-target-repo-runners`.
- Post-merge deploy observation; owned by `runx-post-merge-outcome-observer`.
- Nitrosend-specific copy, labels, and channel names beyond fixtures.

## Dependencies

- Coordinates with `runx-target-repo-runners`.
- Coordinates with `runx-post-merge-outcome-observer`.
- Feeds `rust-nitrosend-dogfood` and `rust-aster-runtime-cutover`.

## Assumptions

- YAML or JSON is acceptable if schema validation and readable error output are
  strong. Choose the format that best matches existing runx policy conventions.
- Aster can initially consume a static policy read model before a full admin UI
  editor exists.

## Touchpoints

- Policy config loader and schema files.
- Issue-intake and issue-to-PR skill contracts.
- CLI command options that accept policy paths.
- Runtime execution context.
- Aster/admin readback docs.

## Risks

- A permissive default could authorize automation in the wrong repo.
- Config drift between runx core and adopter wrappers could recreate the
  current duplication.
- Overfitting to Nitrosend would make the core policy less reusable.

## Acceptance

Profile: strict

Validation:
- `pnpm test`
- `cargo test --manifest-path crates/Cargo.toml`
- policy schema validation fixture command
- `git diff --check`

Required behavior:
- [ ] Unknown target repo fails policy validation.
- [ ] Unknown runner fails policy validation.
- [ ] Missing source-thread routing fails when Slack/GitHub follow-up publishing
  is enabled.
- [ ] Owner routing is explicit for each target or target class.
- [ ] Dedupe strategy is explicit for PR-producing flows.
- [ ] Outcome strategy is explicit for merged, closed-unmerged, failed verify,
  and superseded states.
- [ ] Policy projection redacts secrets and local paths.
- [ ] Nitrosend-like fixture can express existing API/App/workspace routing with
  no custom parser.

## Phase 1: Schema

Status: pending
Dependencies: none

Objective: Make operational policy explicit and validated.

Changes:
- Add versioned schema.
- Add parser and typed validation errors.
- Add positive and negative fixtures.

Acceptance:
- [ ] Fixture policies validate or fail with stable errors.

## Phase 2: Core Consumption

Status: pending
Dependencies: Phase 1

Objective: Make issue-intake and issue-to-PR flows consume policy through one
shared loader.

Changes:
- Replace duplicated policy parsing with core helper.
- Thread policy context into runner and outcome decisions.

Acceptance:
- [ ] Existing skills can read policy through the shared helper.

## Phase 3: Readback Surface

Status: pending
Dependencies: Phase 2

Objective: Expose policy safely to operators and Aster.

Changes:
- Add safe projection/readback command or runtime API.
- Document the admin-visible fields.

Acceptance:
- [ ] Projection includes routing, owners, allowed runners, and outcomes.
- [ ] Projection excludes secrets and absolute local paths.

## Rollback

- Keep existing repo-local policy files valid during implementation, but remove
  duplicated parsing once the core loader is adopted. No legacy alias path should
  remain after cutover.

## Review

Status: not_started
Verdict: none

Findings:
- none

## Self Eval

- Target score: 9.5. Passing means operators can understand and audit what runx
  is allowed to do before it does it.

## Deviations

- none

## Metadata

- created_by: scafld
- planning_reason: move reusable production routing/policy into runx core

## Origin

Created by: scafld
Source: plan

## Harden Rounds

- none

## Planning Log

- 2026-05-19: Expanded placeholder into policy-config contract after Nitrosend
  production dogfood review.
