---
spec_version: '2.0'
task_id: runx-alert-dev-escalation-v1
created: '2026-05-27T15:02:28Z'
updated: '2026-05-27T16:34:28Z'
status: cancelled
harden_status: in_progress
size: medium
risk_level: high
---

# Alert Triage And Dev Escalation Lane

## Current State

Status: cancelled
Current phase: planning
Next: done
Reason: cancel
Blockers: none
Allowed follow-up command: `none`
Latest runner update: none
Review gate: not_started

## Summary

Turn production/system/Sentry-style alerts into governed operational decisions:
monitor/no-action, attach duplicate, create issue, build fix, write a work plan,
or escalate to dev with a precise decision packet.

This lane must be strict. Alert sources are noisy, partial, and often
duplicative. runx should not build code from a truncated alert packet or open
parallel issues for the same fingerprint. It should first verify that the
source is admitted, context is hydrated enough, dedupe has been checked, and
the human-facing next step is explicit.

Suggested subagent ownership: Mendel.

## Objectives

- Define provider-neutral alert evidence requirements without binding runx to
  Sentry, Slack, email, CloudWatch, or any Nitrosend alert implementation.
- Add alert policy filters for:
  - environment;
  - unresolved/regressed status;
  - severity;
  - frequency;
  - affected count;
  - deploy/commit correlation;
  - known duplicate fingerprint;
  - hydration completeness.
- Emit one primary action decision: monitor/no-action, duplicate attach,
  create-issue, build-fix, work-plan, dev-escalation, incident-escalation, or
  manual-review.
- Make dev escalation first-class and dense: problem, evidence, likely owner
  route, likely root cause or competing hypotheses, current links, attempted
  automation, and exact human decision required.
- Preserve source-thread story hooks without leaking raw alert payloads.
- Prove negative cases before live dogfood.

## Scope

In scope:

- New generic sibling skills:
  - `skills/alert-triage/SKILL.md`;
  - `skills/alert-triage/X.yaml`;
  - `skills/dev-escalation/SKILL.md`;
  - `skills/dev-escalation/X.yaml`.
- Docs:
  - `docs/operational-intelligence.md`;
  - `docs/developer-issue-inbox.md`;
  - `docs/thread-story-contract.md`.
- Reusable helper touchpoints:
  - `packages/core/src/source/index.ts`;
  - `packages/core/src/knowledge/thread-story.ts`;
  - contract helpers only when `runx-operational-contracts-v1` promotes them.
- Fixtures:
  - `fixtures/operational-intelligence/alert-triage/**`;
  - `fixtures/runtime/skills/alert-triage/**`;
  - `fixtures/runtime/skills/dev-escalation/**`.
- Tests for policy filtering, dedupe, hydration gates, dev escalation content,
  story redaction, and action selection.

Out of scope:

- Live Sentry API fetches, Slack fetches, CloudWatch fetches, or production
  alert credentials.
- Nitrosend channel names, Sentry project ids, route owners, GitHub labels, or
  deployment hooks.
- Auto-opening incidents in external systems.
- Auto-merging PRs or applying production fixes without human gates.
- Generic support response and outreach behavior except where alert evidence
  becomes an input signal for those sibling lanes.

## Dependencies

- `runx-operational-contracts-v1` for alert, decision, escalation, and action
  packet semantics. Phase 3 may not start until that spec has approved or
  otherwise recorded stable `dev_escalation` and `operational_action_plan`
  shapes.
- `runx-operational-story-outbox-v1` for alert/dev milestone rendering.
- Existing issue and work lanes are dependencies only. This child may delegate
  action packets to them, but it must not alter their behavior:
  - `skills/issue-intake/SKILL.md`;
  - `skills/issue-to-pr/SKILL.md`;
  - `skills/work-plan/SKILL.md`;
  - `docs/issue-to-pr.md`;
  - `docs/developer-issue-inbox.md`.
- Nitrosend integration child for live system-alert/Sentry-style dogfood.

## Assumptions

- A consuming adapter supplies redacted alert evidence and safe source locators.
- An alert with incomplete stack/context may still become monitor/manual-review,
  but it must not become build-fix unless enough evidence exists for a bounded
  change.
- Duplicate fingerprints should attach to an existing harness/change-set/issue
  instead of opening a parallel PR.
- A dev escalation is not a failure mode; it is the right outcome when the
  exact human decision is clearer than an automated mutation.
- Alert-to-PR requires the same human merge gate as issue-to-PR.

## Decisions

- `skill_ownership`: create sibling `alert-triage` and `dev-escalation` skills.
- `alert-triage` selects monitor, duplicate, issue/build/work-plan, escalation,
  or manual-review.
- `dev-escalation` formats the precise human decision packet; it does not open
  PRs or incidents by itself.

## Touchpoints

- `skills/alert-triage/SKILL.md`
- `skills/alert-triage/X.yaml`
- `skills/dev-escalation/SKILL.md`
- `skills/dev-escalation/X.yaml`
- `docs/operational-intelligence.md`
- `docs/developer-issue-inbox.md`
- `docs/thread-story-contract.md`
- `packages/core/src/source/index.ts`
- `packages/core/src/source/index.test.ts`
- `packages/core/src/knowledge/thread-story.ts`
- `packages/core/src/knowledge/index.ts`
- `fixtures/operational-intelligence/alert-triage/**`
- `fixtures/runtime/skills/alert-triage/**`
- `fixtures/runtime/skills/dev-escalation/**`

## Risks

- Alert noise becomes dev noise. Mitigation: strict source policy, thresholds,
  dedupe, and monitor/no-action decisions are first-class.
- Truncated context creates bad PRs. Mitigation: hydration completeness blocks
  build-fix and explains the missing evidence.
- Dev escalation lacks actionability. Mitigation: escalation packet requires
  exact human decision, evidence refs, likely owner route, and next action.
- Provider leakage. Mitigation: fixture inputs include raw-looking payloads while
  expected public outputs live under a separate `public/` fixture subtree that is
  checked for leaks.
- Runtime mutation creep. Mitigation: issue/PR creation is delegated to
  governed lanes; this child selects action and prepares packets.

## Acceptance

Profile: standard

Validation:
- `scafld harden runx-alert-dev-escalation-v1 --provider claude`
- `scafld validate runx-alert-dev-escalation-v1`
- `pnpm typecheck`
- `pnpm test:fast`
- `pnpm boundary:check`
- If contracts change:
  `pnpm exec vitest run --config vitest.fast.config.ts packages/contracts/src`
- If runtime skill fixtures change:
  `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features cli-tool --test integration -- alert_triage dev_escalation`

## Phase 1: Harden Alert And Escalation Boundary

Status: pending
Dependencies: none

Objective: Confirm the skill split, alert policy filters, and escalation packet
before implementation.

Changes:
- Run Claude hardening.
- Confirm `alert-triage` and `dev-escalation` remain separate sibling skills.
- Define the required alert evidence fields and blocked/missing-context states.
- Define dev escalation required fields and exact human gate copy.

Acceptance:
- [ ] `p1_ac1` command - Skill ownership decision is recorded.
  - Command: `sh -c 'spec=$(find .scafld/specs/drafts .scafld/specs/approved .scafld/specs/active -name runx-alert-dev-escalation-v1.md -print -quit); test -n "$spec" && rg -n "skill_ownership.*alert-triage.*dev-escalation|does not open PRs" "$spec"'`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `p1_ac2` command - Spec validates after hardening edits.
  - Command: `scafld validate runx-alert-dev-escalation-v1`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Phase 2: Fixtures And Expected Decisions

Status: pending
Dependencies: Phase 1

Objective: Lock down alert behavior with deterministic cases.

Changes:
- Add fixture cases for:
  - noisy/non-production alert denied or monitor-only;
  - production unresolved/regressed alert with complete context;
  - repeated duplicate alert attaches to existing harness;
  - truncated payload blocks build-fix;
  - alert that should create issue but not PR;
  - alert that should build a bounded fix;
  - alert that should escalate to dev/incident;
  - resolved alert that should close/observe only.
- Add expected outputs with evidence refs, dedupe ids, source thread locator,
  action plan, and public story hints.
- Split fixtures into `private/` source inputs and `public/` expected decision
  and story outputs; raw-looking provider fields are allowed only in `private/`.

Acceptance:
- [ ] `p2_ac1` command - Alert fixtures cover trigger and non-trigger cases.
  - Command: `test -d fixtures/operational-intelligence/alert-triage/public && for token in monitor duplicate build_fix dev_escalation manual_review truncated; do rg -n "$token" fixtures/operational-intelligence/alert-triage >/dev/null || exit 1; done`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `p2_ac2` command - Public fixture output avoids raw alert/provider data.
  - Command: `sh -c 'test -d fixtures/operational-intelligence/alert-triage/public && if rg -n "xox[baprs]-|BEGIN .*PRIVATE KEY|/Users/|url_private_download|raw_payload|SENTRY_AUTH_TOKEN" fixtures/operational-intelligence/alert-triage/public; then exit 1; fi'`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Phase 3: Lane Implementation

Status: pending
Dependencies: Phase 2

Objective: Implement alert policy evaluation, dedupe, and escalation reporting.

Changes:
- Add or update the selected skill/lane.
- Evaluate environment, status, severity, frequency, affected count, deploy
  correlation, dedupe, and hydration completeness.
- Emit `operational_decision`, `operational_action_plan`, and optional
  `dev_escalation`.
- Delegate issue/build/work-plan outcomes to existing lanes through packets.
- Add tests for all fixture decisions and redaction.

Acceptance:
- [ ] `p3_ac1` command - TypeScript compiles.
  - Command: `pnpm typecheck`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `p3_ac2` command - Fast tests pass.
  - Command: `pnpm test:fast`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `p3_ac3` command - Boundary checks pass.
  - Command: `pnpm boundary:check`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Phase 4: Story And Integration Contract

Status: pending
Dependencies: Phase 3

Objective: Make alert followups readable and actionable on source threads.

Changes:
- Add story hints for accepted, denied/no-action, duplicate attached, issue
  recommended, PR recommended, escalation, human gate, and outcome.
- Ensure escalation copy names what the human needs to decide.
- Add sanitized integration fixture contract for consuming repos.
- Verify source-thread-required policy fails closed when no safe locator exists.

Acceptance:
- [ ] `p4_ac1` command - Story vocabulary includes alert and escalation gates.
  - Command: `for token in "dev escalation" "duplicate attached" "human decision"; do rg -n "$token" skills/alert-triage skills/dev-escalation packages/core/src/knowledge >/dev/null || exit 1; done`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `p4_ac2` command - Runtime skill fixtures pass when added.
  - Command: `test -d fixtures/runtime/skills/alert-triage && test -d fixtures/runtime/skills/dev-escalation && cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features cli-tool --test integration -- alert_triage dev_escalation`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Rollback

- Remove alert-triage/dev-escalation skill files, fixtures, docs, and helper
  tests introduced by this child.
- Revert shared source/story helper edits only if they are not used by sibling
  specs.
- Do not change live alert providers, Slack channels, issue/PR state, or
  production deployments as part of rollback.

## Review

Status: not_started
Verdict: none

Findings:
- none

Required gates:
- Draft hardening before approval:
  `scafld harden runx-alert-dev-escalation-v1 --provider claude`
- Completion review after implementation:
  `scafld review runx-alert-dev-escalation-v1 --provider claude`
- `--provider local` is not sufficient for completion.

## Self Eval

- Pending implementation. Target bar: noisy alerts are filtered, actionable
  alerts route cleanly, truncated context blocks mutation, duplicates attach,
  and dev escalation is specific enough for a human to decide immediately.

## Deviations

- none

## Metadata

- created_by: scafld
- parent_spec: runx-operational-intelligence-action-layer-v1
- suggested_subagent: Mendel

## Origin

Created by: scafld
Source: plan

## Harden Rounds

### round-1

Status: in_progress
Started: 2026-05-27T15:31:18Z
Ended: none

Checks:
- none

Issues:
- none


## Planning Log

- Split from the parent operational-intelligence program spec.
- This child deliberately does not depend on live Sentry access; live dogfood
  belongs to the Nitrosend integration child.
