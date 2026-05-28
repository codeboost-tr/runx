---
spec_version: '2.0'
task_id: runx-support-triage-response-v1
created: '2026-05-27T15:02:28Z'
updated: '2026-05-27T16:34:28Z'
status: cancelled
harden_status: in_progress
size: medium
risk_level: high
---

# Support Triage And Response Lane

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

Extend runx from issue intake into a support-first operational lane. The lane
must decide the next useful action for an admitted support-like source event:
answer with a draft, ask for missing evidence, attach to an existing issue,
create an issue, build a bounded fix, write a work plan, escalate to dev,
decline/no-action, or require manual review.

The most important constraint is trust. A support reply draft is not a sent
reply. runx may prepare a customer-safe draft and reasoning packet, but the
consuming product controls the send action, support-channel UX, customer
identity, and account enrichment.

Suggested subagent ownership: Mendel.

## Objectives

- Add a generic support/action lane that works from portable source and context
  packets, not from live Slack/support-tool credentials.
- Classify support events into actionable categories:
  - question;
  - account access;
  - billing/account;
  - deliverability;
  - integration/API;
  - docs/onboarding;
  - feature request;
  - product bug;
  - incident/regression;
  - other/manual-review.
- Emit one primary action plan with clear rationale and evidence refs.
- Generate reply drafts for answer-now and ask-for-info cases without marking
  anything as sent.
- Reuse existing issue-intake/issue-triage outputs where possible instead of
  duplicating issue-to-PR logic.
- Preserve source-thread story hooks for the story/outbox child to render.
- Prove negative cases: insufficient context, account/billing hold, duplicate
  issue, unsafe customer copy, and high-risk manual review.

## Scope

In scope:

- New generic sibling skill/lane:
  - `skills/support-response/SKILL.md`;
  - `skills/support-response/X.yaml`.
- Support-oriented examples in:
  - `docs/operational-intelligence.md`;
  - `docs/developer-issue-inbox.md`;
  - `docs/thread-story-contract.md` after story milestones exist.
- Core helpers only when reusable:
  - `packages/core/src/source/index.ts`;
  - `packages/core/src/knowledge/thread-story.ts`;
  - `packages/core/src/knowledge/index.ts`.
- Fixtures:
  - `fixtures/operational-intelligence/support-response/**`;
  - `fixtures/runtime/skills/support-response/**` when runtime fixtures are
    required.
- Tests for classification, action selection, missing context, duplicate reuse,
  no-send semantics, and public story safety.

Out of scope:

- Live Slack, Zendesk, Intercom, HelpScout, email, CRM, billing, or customer
  API integrations.
- Sending customer replies or support messages.
- Product-specific support macros, owner names, account ids, customer emails,
  Slack channels, or Nitrosend-specific copy.
- Auto-merging PRs, mutating billing state, or changing customer data.
- Replacing `issue-to-pr`; this lane delegates to issue/build/work-plan lanes
  only when evidence and policy warrant it.

## Dependencies

- `runx-operational-contracts-v1` for packet names and authority semantics.
  Phase 3 may not start until that spec has approved or otherwise recorded the
  stable shapes for `operational_action_plan` and `support_reply_draft`.
- `runx-operational-story-outbox-v1` for public milestone rendering.
- Existing issue flow docs and skills:
  - `docs/developer-issue-inbox.md`;
  - `docs/issue-to-pr.md`;
  - `skills/issue-intake/SKILL.md`;
  - `skills/issue-triage/SKILL.md`;
  - `skills/issue-to-pr/SKILL.md`;
  - `skills/work-plan/SKILL.md`.
- Nitrosend integration child for live support-channel policy and dogfood:
  - `nitrosend-operational-intelligence-integration-v1`.

## Assumptions

- A provider adapter supplies a redacted support context packet that includes
  source text, thread locator when safe, account/customer summary when allowed,
  evidence refs, and redaction status.
- If account/billing/private data is required and absent, the lane asks for
  missing context or sends manual-review, not build-fix.
- If a support report is a concrete product bug, the lane may recommend
  create-issue/build-fix, but the mutation happens through existing governed
  issue and PR lanes.
- Duplicate support reports attach to an existing harness/change-set/issue
  when one is known.
- The lane can be useful without creating code: answer now, ask for info,
  update docs recommendation, no-action, or dev escalation are valid outcomes.

## Decisions

- `skill_ownership`: create a sibling `support-response` skill.
- `issue-intake` remains the source admission and lane-selection input.
- `support-response` is the canonical executor for `recommended_lane=reply-only`
  when the admitted source is support-like; no `recommended_lane` enum expansion
  is required in this child.
- `issue-intake` may receive only a bounded routing pointer/example that maps
  support-like `reply-only` sources to `support-response`. It must not gain a new
  reasoning section, second artifact, or expanded lane enum.
- `issue-triage` remains thread discovery and existing issue-response support; it
  is a dependency, not a behavioral touchpoint for this child.
- This child must not emit a second `intake_report`-shaped artifact.
- The contracts child owns the `docs/operational-intelligence.md` skeleton; this
  child adds only the support-response section when that doc exists.

## Touchpoints

- `skills/support-response/SKILL.md`
- `skills/support-response/X.yaml`
- `skills/issue-intake/SKILL.md` (routing pointer/example only)
- `skills/issue-intake/X.yaml` (routing fixture only)
- `docs/operational-intelligence.md`
- `docs/developer-issue-inbox.md`
- `docs/thread-story-contract.md`
- `packages/core/src/source/index.ts`
- `packages/core/src/source/index.test.ts`
- `packages/core/src/knowledge/thread-story.ts`
- `packages/core/src/knowledge/index.ts`
- `fixtures/operational-intelligence/support-response/**`
- `fixtures/runtime/skills/support-response/**`
- relevant skill/runtime tests only after hardening confirms exact harness

## Risks

- Customer copy risk. Mitigation: reply drafts are explicitly unsent and must
  carry approval requirements and caveats.
- Over-triggering issues/PRs from support noise. Mitigation: create issue or
  build fix only when evidence is concrete, deduped, and policy-approved.
- Missing private context. Mitigation: ask-for-info/manual-review beats
  speculative fixes.
- Duplicating issue-to-PR. Mitigation: this lane selects actions; issue-to-PR
  still owns governed code mutation.
- Provider lock-in. Mitigation: input is a portable source/context packet, not
  a Slack or support API payload.
- Noisy public story. Mitigation: public output is a short status, evidence
  bullets, draft/next action, and links; raw reasoning and artifacts remain
  behind receipts.

## Acceptance

Profile: standard

Validation:
- `scafld harden runx-support-triage-response-v1 --provider claude`
- `scafld validate runx-support-triage-response-v1`
- `pnpm typecheck`
- `pnpm test:fast`
- `pnpm boundary:check`
- If contracts change:
  `pnpm exec vitest run --config vitest.fast.config.ts packages/contracts/src`
- If runtime skill fixtures change:
  `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features cli-tool --test integration -- support_response`

## Phase 1: Harden Lane Boundary

Status: pending
Dependencies: none

Objective: Confirm the sibling support-response boundary and its invocation path.

Changes:
- Run Claude hardening.
- Decide exact skill ownership and avoid duplicate reasoning paths.
- Define support categories, action taxonomy, and required input context.
- Define reply-draft safety language and approval semantics.

Acceptance:
- [ ] `p1_ac1` command - Skill ownership decision is recorded.
  - Command: `sh -c 'spec=$(find .scafld/specs/drafts .scafld/specs/approved .scafld/specs/active -name runx-support-triage-response-v1.md -print -quit); test -n "$spec" && rg -n "skill_ownership.*support-response|canonical executor for.*reply-only|second .?intake_report" "$spec"'`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `p1_ac2` command - Spec validates after hardening edits.
  - Command: `scafld validate runx-support-triage-response-v1`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Phase 2: Fixtures And Prompt Contract

Status: pending
Dependencies: Phase 1

Objective: Make the desired behavior deterministic before implementation.

Changes:
- Add fixture cases for:
  - answer-now support question;
  - ask for missing evidence;
  - duplicate attaches to existing issue/harness;
  - concrete bug routes to create-issue/build-fix;
  - docs/onboarding gap routes to docs/support macro recommendation;
  - account/billing case requires manual review;
  - unsafe customer copy becomes blocked/manual-review;
  - low-confidence/no-action.
- Define expected output shape for each fixture.
- Include raw-looking provider/customer content in private fixture inputs and
  assert public output is redacted.
- Split fixtures into `private/` source inputs and `public/` expected outputs.
  Raw-looking provider/customer fields and already-sent evidence are allowed only
  in `private/`.
- Include a `missing_thread_locator` fixture where policy requires
  reply-in-thread; expected public output must be blocked/manual-review with no
  root fallback.

Acceptance:
- [ ] `p2_ac1` command - Support fixtures and expected outputs exist.
  - Command: `test -d fixtures/operational-intelligence/support-response/public && for token in reply_draft ask_for_info duplicate build_fix docs_gap account_billing manual_review unsafe_customer_copy no_action missing_thread_locator; do rg -n "$token" fixtures/operational-intelligence/support-response >/dev/null || exit 1; done`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `p2_ac2` command - Public fixtures do not expose obvious secrets or local paths.
  - Command: `sh -c 'test -d fixtures/operational-intelligence/support-response/public && if rg -n "xox[baprs]-|BEGIN .*PRIVATE KEY|/Users/|url_private_download|raw_payload|customer_email" fixtures/operational-intelligence/support-response/public; then exit 1; fi'`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `p2_ac3` command - Reply draft fixtures cannot represent a sent customer message.
  - Command: `sh -c 'test -d fixtures/operational-intelligence/support-response/public && if rg -n "\"sent\"\\s*:\\s*true|delivery_status.*sent|delivered_at" fixtures/operational-intelligence/support-response/public; then exit 1; fi'`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Phase 3: Lane Implementation

Status: pending
Dependencies: Phase 2

Objective: Implement the generic support action selector and draft generator.

Changes:
- Add or update the selected skill/lane.
- Emit one `operational_decision`, one `operational_action_plan`, and optional
  `support_reply_draft`.
- Enforce no-send semantics in the output.
- Route issue/build/work-plan outcomes to existing lane contracts without
  duplicating their implementation.
- Wire the `reply-only` intake result to the sibling `support-response` executor
  through a bounded routing pointer, without expanding the intake lane enum.
- Preserve source ids, dedupe ids, evidence refs, missing-context notes, and
  story milestone hints.

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
- [ ] `p3_ac4` command - Reply-only support routing points to support-response.
  - Command: `rg -n "reply-only.*support-response|support-response.*reply-only" skills/issue-intake skills/support-response`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `p3_ac5` command - Intake lane enum is not widened for support-response.
  - Command: `sh -c 'if rg -n "recommended_lane.*support-response|support-response.*recommended_lane" skills/issue-intake/X.yaml packages/contracts/src/schemas; then exit 1; fi'`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Phase 4: Story And Dogfood Readiness

Status: pending
Dependencies: Phase 3

Objective: Make the support lane useful in Slack/GitHub/support surfaces without
provider-specific code in runx core.

Changes:
- Add story milestone hints for accepted, hydrated, triaged, reply drafted,
  ask-for-info, issue recommended, build-fix recommended, manual-review, and
  no-action.
- Verify public copy is concise and human-readable.
- Add a sanitized dogfood input contract for consuming repos to use.
- Ensure missing thread locator fails closed when policy requires reply-in-thread.

Acceptance:
- [ ] `p4_ac1` command - Story vocabulary is present.
  - Command: `for token in "reply drafted" "ask-for-info" "missing thread locator" "no root fallback"; do rg -n "$token" skills/support-response packages/core/src/knowledge >/dev/null || exit 1; done`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `p4_ac2` command - Runtime skill fixtures pass when added.
  - Command: `test -d fixtures/runtime/skills/support-response && test -f crates/runx-runtime/tests/support_response.rs && grep -q '^mod support_response;' crates/runx-runtime/tests/integration.rs && cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features cli-tool --test integration -- support_response`
  - Expected kind: `exit_code_zero`
  - Status: pending
- [ ] `p4_ac3` command - Missing thread locator case fails closed.
  - Command: `rg -n "missing_thread_locator|no root fallback|blocked.*thread" fixtures/operational-intelligence/support-response/public skills/support-response packages/core/src/knowledge`
  - Expected kind: `exit_code_zero`
  - Status: pending

## Rollback

- Remove the support-response skill, fixtures, docs, and helper tests introduced
  by this child.
- Revert the bounded routing pointer/example in `issue-intake` if introduced.
- Leave issue-intake and issue-to-pr behavior intact unless this spec explicitly
  changed a shared helper.
- No live support messages, customer sends, billing state, or provider data are
  mutated by this child.

## Review

Status: not_started
Verdict: none

Findings:
- none

Required gates:
- Draft hardening before approval:
  `scafld harden runx-support-triage-response-v1 --provider claude`
- Completion review after implementation:
  `scafld review runx-support-triage-response-v1 --provider claude`
- `--provider local` is not sufficient for completion.

## Self Eval

- Pending implementation. Target bar: the lane handles common support work
  before code mutation, produces useful unsent replies, blocks unsafe actions,
  and remains generic enough for non-Nitrosend consumers.

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

Status: needs_revision
Started: 2026-05-27T15:24:46Z
Ended: 2026-05-27T15:24:46Z
Verdict: needs_revision
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: Spec is architecturally coherent for a Mendel-sized child, but two of its phase acceptance commands are too loose to prove the phase work actually landed, Phase 1's "decide skill ownership" leaves no recorded artifact, and several downstream coupling/touchpoint statements need tightening before approve.

Checks:
- path audit
  - Grounded in: code:oss/skills/issue-intake/SKILL.md:1, code:oss/skills/issue-triage/SKILL.md:1, code:oss/docs/thread-story-contract.md:1, code:oss/packages/core/src/knowledge/thread-story.ts
  - Result: passed
  - Evidence: Touchpoints `skills/issue-intake/SKILL.md`, `skills/issue-triage/SKILL.md` (and their X.yaml siblings), `docs/thread-story-contract.md`, `docs/developer-issue-inbox.md`, `docs/issue-to-pr.md`, `packages/core/src/source/index.ts`, `packages/core/src/knowledge/thread-story.ts`, and `packages/core/src/knowledge/index.ts` were all verified to exist. `skills/support-response/**` and `fixtures/operational-intelligence/support-response/**` correctly do not exist yet — they are intentionally future paths created by this spec. `docs/operational-intelligence.md` is also future, but its creation is not assigned to a phase here or clearly delegated.
- command audit
  - Grounded in: code:oss/CLAUDE.md, code:oss/package.json:37
  - Result: passed
  - Evidence: Project-invariant `pnpm verify:fast` is available but the spec correctly picks the narrower gates per phase. `pnpm test:fast` is the additive fast lane per oss/CLAUDE.md.
- scope/migration audit
  - Grounded in: spec_gap:phases.phase1 vs scope.in_scope, code:oss/skills/issue-intake/SKILL.md:1
  - Result: failed
  - Evidence: Scope lists both `skills/support-response/{SKILL.md,X.yaml}` AND the touchpoints list edits to `skills/issue-triage/{SKILL.md,X.yaml}` and `skills/issue-intake/{SKILL.md,X.yaml}` while Phase 1 is supposed to *decide* between sibling skill vs. extension. As written, an implementer is allowed to touch all three trees regardless of the decision, which silently widens scope. The Phase 1 ownership decision is not echoed back into Touchpoints or Scope, so the gate has no enforced effect on the rest of the spec.
- acceptance timing audit
  - Grounded in: code:oss/skills/issue-intake/SKILL.md:17, code:oss/skills/issue-intake/SKILL.md:94, code:oss/docs/issue-to-pr.md, code:oss/docs/thread-story-contract.md
  - Result: failed
  - Evidence: Phase 4 `p4_ac1` (`rg -n "reply drafted|ask-for-info|manual-review|no-action|source-thread" docs packages/core/src/knowledge skills`) is satisfied by pre-existing matches today: `skills/issue-intake/SKILL.md` already contains `manual-review` (lines 17, 27, 44, 58) and `source-thread`; `docs/issue-to-pr.md`, `docs/developer-issue-inbox.md`, `docs/thread-story-contract.md`, and `docs/harness-control-plane.md` already match. The discriminative new strings (`reply drafted`, `ask-for-info`) are not required by the alternation. The check will exit zero with zero Phase 4 work done. Similarly p4_ac2 runs the entire `integration` test binary and does not prove any support-response fixture was added — convention in sibling specs is `--test integration <module_filter>` (see `runx-operational-intelligence-action-layer-v1.md:318`).
- rollback/repair audit
  - Grounded in: spec_gap:rollback vs touchpoints
  - Result: failed
  - Evidence: Rollback says "Revert any changes to existing issue-triage prompts if hardening selected an extension approach" but Touchpoints (lines 134–135) also include `skills/issue-intake/SKILL.md` and `skills/issue-intake/X.yaml`. If extension landed in issue-intake (the more likely target, since issue-intake already owns category/lane/needs_human selection per SKILL.md:50-89), rollback as written would not revert it. Rollback must enumerate every touched existing skill consistent with the Phase 1 decision.
- design challenge
  - Grounded in: code:oss/skills/issue-intake/SKILL.md:1, code:oss/skills/issue-triage/SKILL.md:1, archive:runx-operational-intelligence-action-layer-v1
  - Result: passed
  - Evidence: Splitting support-action selection into its own lane is defensible: `issue-intake` is already the gate that classifies/selects the next lane (issue-to-pr, work-plan, reply-only, manual-review) and `issue-triage` is about thread discovery + response drafting. Adding `support-response` as a sibling that consumes the intake `signal`+`change_set` and emits `operational_decision`/`operational_action_plan`/optional `support_reply_draft` aligns with the parent program's lane taxonomy without duplicating issue-to-PR. The risk is duplicate classification logic with issue-intake, which Phase 1 must explicitly settle (and should also block the spec from emitting a second `intake_report`-shaped artifact).

Issues:
- [high/blocks approval] `harden-1` acceptance - p4_ac1 vocabulary check passes today with zero Phase 4 work because `manual-review`, `no-action`, and `source-thread` already exist in scoped trees.
  - Status: open
  - Grounded in: code:oss/skills/issue-intake/SKILL.md:17, code:oss/docs/thread-story-contract.md, code:oss/docs/issue-to-pr.md
  - Evidence: `rg -n "reply drafted|ask-for-info|manual-review|no-action|source-thread" docs packages/core/src/knowledge skills` matches `skills/issue-intake/SKILL.md` (lines 17, 27, 44, 58, 94, 96, 109, 198) and multiple docs files (`docs/issue-to-pr.md`, `docs/developer-issue-inbox.md`, `docs/thread-story-contract.md`, `docs/harness-control-plane.md`). The alternation does not require the discriminative new strings (`reply drafted`, `ask-for-info`) to appear. The acceptance command is a false positive gate.
  - Recommendation: Split the check into two greps that must each return ≥1 match in a path the spec actually creates, e.g. `rg -n "reply drafted" docs/operational-intelligence.md skills/support-response packages/core/src/knowledge` AND `rg -n "ask-for-info" docs/operational-intelligence.md skills/support-response packages/core/src/knowledge`. Optionally add a third grep for `no-action` and `manual-review` scoped to the new skill path only.
  - Question: Should the milestone-vocabulary AC be tightened to require evidence in `skills/support-response/` and the new `docs/operational-intelligence.md` (or whichever doc this child owns), instead of any-file matches?
  - Recommended answer: Yes. Scope p4_ac1 to the new skill path and the doc this spec owns, and assert the discriminative strings (`reply drafted`, `ask-for-info`) individually.
  - If unanswered: Default to scoping the grep to `skills/support-response packages/core/src/knowledge docs/operational-intelligence.md` and requiring `reply drafted` and `ask-for-info` as separate ACs.
- [medium/blocks approval] `harden-2` acceptance - p4_ac2 runs the whole integration test binary and does not prove support-response runtime fixtures exist.
  - Status: open
  - Grounded in: code:oss/crates/runx-runtime/tests/integration.rs:1, code:oss/.scafld/specs/drafts/runx-operational-intelligence-action-layer-v1.md:318
  - Evidence: The runx-runtime `integration` binary already passes today and would pass without any Phase 4 fixture being added. The parent program spec uses the scoped form `--test integration skill_issue_intake skill_issue_to_pr thread_outbox_provider` (line 318). The child spec drops the module filter, so the check is not coupled to the change.
  - Recommendation: Either (a) add `--test integration support_response` (mirroring the parent convention and requiring a `support_response.rs` module under tests/) plus a precondition `test -f crates/runx-runtime/tests/support_response.rs`, or (b) follow it with a fixture-presence check like `test -d fixtures/runtime/skills/support-response && test -n "$(ls fixtures/runtime/skills/support-response)"`.
  - Question: Do support-response runtime fixtures land in this spec at all, or are they deferred to a later runtime-coverage spec? The current p4_ac2 makes that ambiguous.
  - Recommended answer: If runtime fixtures land here, add a scoped module filter + fixture-presence check. If not, remove p4_ac2 and rely on fixtures/operational-intelligence/ asserted in Phase 2.
- [medium/blocks approval] `harden-3` scope - Phase 1 must resolve sibling-vs-extension ownership, but the decision has no recorded artifact and Touchpoints/Scope keep both paths live.
  - Status: open
  - Grounded in: spec_gap:phases.phase1, spec_gap:touchpoints, code:oss/skills/issue-intake/SKILL.md:50
  - Evidence: Phase 1 acceptance is only `scafld harden ... --provider claude` and `scafld validate ...`. There is no AC that records the chosen approach (e.g. a Decision section in the spec, or a presence check for either `skills/support-response/SKILL.md` xor an edit marker in `skills/issue-intake/SKILL.md`). Touchpoints include both `skills/support-response/*` AND `skills/issue-{intake,triage}/{SKILL.md,X.yaml}`, which leaves implementers room to touch all three even after the decision.
  - Recommendation: Add a Phase 1 Decisions block to the spec body that this spec must edit, and add a p1_ac3 that greps for the decision (e.g. `rg -n "Skill ownership decision:" .scafld/specs/active/runx-support-triage-response-v1.md`). After the decision, narrow Touchpoints to the chosen tree only; relegate the unchosen tree to Out of Scope.
  - Question: Will Phase 1's outcome be recorded as a Decision block in this spec body (so future phases and rollback are unambiguous about which skill tree is mutated)?
  - Recommended answer: Yes — add `## Decisions` with `skill_ownership: sibling | extend_issue_intake | extend_issue_triage` plus a p1_ac3 grep, and prune Touchpoints to match before approve.
- [medium/advisory] `harden-4` invariant - No-send semantics is asserted in prose but has no acceptance check tying it to the contract or fixture shape.
  - Status: open
  - Grounded in: spec_gap:phase3.changes "Enforce no-send semantics"
  - Evidence: Phase 3 says "Enforce no-send semantics in the output" but its ACs are only `pnpm typecheck`, `pnpm test:fast`, `pnpm boundary:check`. There is no fixture-key check or contract-shape test that proves the output cannot represent a sent reply (e.g. forbidding a `sent: true` or `delivery_status: sent` field on `support_reply_draft`).
  - Recommendation: Add a Phase 2 fixture-key assertion: `! rg -n "\"sent\"\\s*:\\s*true|delivery_status.*sent|delivered_at" fixtures/operational-intelligence/support-response`. If `support_reply_draft` becomes a public contract, also gate it in the contract test in runx-operational-contracts-v1.
- [low/advisory] `harden-5` rollback - Rollback mentions issue-triage but not issue-intake; both are in Touchpoints.
  - Status: open
  - Grounded in: spec_gap:rollback vs touchpoints
  - Evidence: Rollback line 295 says "Revert any changes to existing issue-triage prompts if hardening selected an extension approach." Touchpoints (lines 134–135) also list `skills/issue-intake/SKILL.md` and `skills/issue-intake/X.yaml`. issue-intake is the more likely extension target given its existing category/lane/needs_human role (SKILL.md:50-89).
  - Recommendation: Rewrite rollback to read "Revert any changes to existing `issue-intake` or `issue-triage` prompts and X.yaml files if hardening selected an extension approach," then have Phase 1's Decision block narrow the rollback to the actual touched skill.
- [low/advisory] `harden-6` ownership - `docs/operational-intelligence.md` is a touchpoint but its creation isn't owned by any phase here or unambiguously by the contracts child.
  - Status: open
  - Grounded in: spec_gap:touchpoints `docs/operational-intelligence.md`
  - Evidence: Touchpoints lists `docs/operational-intelligence.md` (line 137). The parent program spec and `runx-operational-contracts-v1` both reference it as `new docs/operational-intelligence.md if the concepts need one home`. p4_ac1 also greps `docs` for milestone vocabulary that this doc is the natural home for. If `runx-operational-contracts-v1` declines to create it, this spec is left orphaned.
  - Recommendation: State explicitly which child creates `docs/operational-intelligence.md`. Either (a) declare this spec creates it in Phase 4 and add `test -f docs/operational-intelligence.md` to p4_ac1, or (b) move the path to `runx-operational-contracts-v1` and add a Dependencies note that the contracts child must land first.
  - Question: Which spec owns the creation of `docs/operational-intelligence.md` — this support child or the contracts child?
  - Recommended answer: The contracts child should own the doc skeleton; this spec adds the support-response section only.
- [medium/advisory] `harden-7` dependency - Phase 3 emits `operational_decision`, `operational_action_plan`, and `support_reply_draft` but the contracts child that defines these shapes is only listed as a Dependency without an ordering guarantee.
  - Status: open
  - Grounded in: spec_gap:dependencies vs phase3
  - Evidence: Dependencies (line 101) lists `runx-operational-contracts-v1` for packet names, but the spec does not say this child must wait for contracts-v1 to be approved (or at least to have stabilized internal helper shapes). If both are in flight in parallel, Phase 3 will land emitter code against unstable shapes and `pnpm test:fast` will not catch a missing contract.
  - Recommendation: Add a Dependencies acceptance gate or sequencing note: "Phase 3 may not start until `runx-operational-contracts-v1` has either (a) reached approved status, or (b) declared the relevant packets as internal helpers with stable shapes documented in `docs/operational-intelligence.md`." Optionally add a p3_ac0 that asserts the packet types compile-import successfully from the contracts child.

### round-2

Status: needs_revision
Started: 2026-05-27T15:48:15Z
Ended: 2026-05-27T15:48:15Z
Verdict: needs_revision
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: Round-2 improvements landed: the skill_ownership Decision is now recorded, p1_ac1 binds it, p2_ac3 forbids "sent" semantics, and p4_ac1 was scoped + split. Three blockers remain: (1) the sibling `support-response` skill has no defined invocation path because `issue-intake`'s `recommended_lane` enum only allows `issue-to-pr`/`work-plan`/`reply-only`/`manual-review` — there is no architectural seam for the new lane; (2) Touchpoints still list edits to `skills/issue-intake/{SKILL.md,X.yaml}` and `skills/issue-triage/{SKILL.md,X.yaml}` which contradicts the sibling-only decision unless the spec explicitly carves out a tiny routing-pointer change; (3) p4_ac2's `-- support_response` cargo filter matches zero tests unless `mod support_response;` is added to `crates/runx-runtime/tests/integration.rs`, so the gate is one config edit away from being silently inert. Two advisories: p2_ac1's alternation under-specifies the eight declared fixture categories, and Phase 4's "fail-closed when policy requires reply-in-thread" rule has no AC.

Checks:
- path audit
  - Grounded in: code:oss/skills/issue-intake/SKILL.md, code:oss/skills/issue-triage/SKILL.md, code:oss/packages/core/src/source/index.ts, code:oss/packages/core/src/knowledge/thread-story.ts, code:oss/packages/core/src/knowledge/index.ts, code:oss/docs/thread-story-contract.md, code:oss/docs/developer-issue-inbox.md
  - Result: passed
  - Evidence: Verified existing touchpoints: skills/issue-intake/{SKILL.md,X.yaml}, skills/issue-triage/SKILL.md, packages/core/src/source/index.ts, packages/core/src/knowledge/{thread-story.ts,index.ts}, docs/thread-story-contract.md, docs/developer-issue-inbox.md. Future touchpoints not yet created (correct for a draft): skills/support-response/**, fixtures/operational-intelligence/support-response/**, fixtures/runtime/skills/support-response/**, docs/operational-intelligence.md. The Decisions section now explicitly says the contracts child owns docs/operational-intelligence.md, addressing round-1 harden-6.
- command audit
  - Grounded in: code:oss/package.json:33-37, code:oss/crates/runx-runtime/tests/integration.rs:1-50
  - Result: passed
  - Evidence: pnpm typecheck (line 33), pnpm test:fast (line 35), pnpm boundary:check (line 37) all resolve. cargo invocation matches project pattern in package.json:53/63/64/65 where `--test integration -- <module_filter>` is the canonical form. The integration binary is a single-binary aggregator (integration.rs:1-50), so a `support_response` filter requires a corresponding `mod support_response;` line to exist before tests can match — see issue harden-r2-3.
- scope/migration audit
  - Grounded in: code:oss/skills/issue-intake/SKILL.md:57, code:oss/skills/issue-intake/X.yaml:260-317, spec_gap:decisions vs touchpoints
  - Result: failed
  - Evidence: Decisions (line 132) commits to a sibling `support-response` skill and asserts `issue-intake remains the source admission and lane-selection input`. But issue-intake's `recommended_lane` enum is fixed to `issue-to-pr | work-plan | reply-only | manual-review` (SKILL.md:57, X.yaml:273/286/317). No documented invocation seam exists for `support-response`. Simultaneously, Touchpoints (lines 144-147) still list `skills/issue-intake/{SKILL.md,X.yaml}` and `skills/issue-triage/{SKILL.md,X.yaml}` as in-scope edits with no acceptance check pinning what minimal change is allowed. Either the spec must (a) widen `recommended_lane` and edit intake's contract + X.yaml fixtures, or (b) declare that support-response is invoked as the executor of `recommended_lane=reply-only` and route via an orchestrator the spec names. As written, the integration is undefined and Touchpoints permit unbounded edits to intake/triage trees.
- acceptance timing audit
  - Grounded in: code:oss/crates/runx-runtime/tests/integration.rs:8-50, spec_gap:phase4 p4_ac2, spec_gap:phase2 p2_ac1
  - Result: failed
  - Evidence: p4_ac2 = `test -d fixtures/runtime/skills/support-response && cargo test ... --test integration -- support_response`. The `-d` precondition fixes the round-1 vacuous-pass on the fixture directory, but the `-- support_response` filter runs against integration.rs which currently has no `support_response` module (only `mod support;` a shared helper, lines 8-50). cargo test exits 0 with `0 tests run` if no test names match — so adding fixtures alone is enough to make the gate green without ever exercising the lane. The check needs `test -f crates/runx-runtime/tests/support_response.rs` (or a grep for `mod support_response;` in integration.rs) as a structural precondition. Separately, p2_ac1's `rg -n 'reply_draft|ask_for_info|manual_review|build_fix|duplicate'` is an alternation that any one match satisfies, but Phase 2 enumerates eight fixture categories (answer-now, ask-for-info, duplicate, concrete-bug, docs/onboarding gap, account/billing manual review, unsafe customer copy, low-confidence/no-action); the gate only weakly proves five and only requires one.
- rollback/repair audit
  - Grounded in: spec_gap:rollback vs touchpoints
  - Result: passed
  - Evidence: Rollback (lines 305-314) now names both `issue-intake` and `issue-triage` prompt + X.yaml reversion paths, addressing round-1 harden-5. With the sibling decision recorded in Decisions (line 132), the rollback is internally consistent — but only if the scope/migration issue above is also resolved by pruning Touchpoints or scoping the allowed intake edit. Rollback also correctly states no live support sends, customer mutations, or billing state are mutated, which matches the no-send invariant in p2_ac3.
- design challenge
  - Grounded in: code:oss/skills/issue-intake/SKILL.md:1-89, code:oss/skills/issue-triage/SKILL.md:1-65, archive:runx-operational-intelligence-action-layer-v1
  - Result: passed
  - Evidence: A sibling lane is the right architectural call: issue-intake is a classification + lane-selector that produces `intake_report` + `change_set` + `signal`; issue-triage is a discovery/response drafter against a chosen thread. A support-response skill that takes a stabilized intake/change_set and emits `operational_decision` + `operational_action_plan` + optional `support_reply_draft` does not duplicate either. It is not a bandaid (the issue-to-PR/work-plan lanes do not cover ask-for-info, account/billing hold, or unsent draft semantics) and not future bloat (the spec explicitly defers Slack/Zendesk/Intercom adapters and customer-send authority). The unresolved architectural risk is purely the integration seam from intake → support-response (see scope/migration audit) — settle that and the design is sound.

Issues:
- [high/blocks approval] `harden-r2-1` scope - Sibling `support-response` skill has no defined invocation path from issue-intake's lane enum.
  - Status: open
  - Grounded in: code:oss/skills/issue-intake/SKILL.md:57, code:oss/skills/issue-intake/X.yaml:260-317
  - Evidence: Decisions (line 132) chooses a sibling skill and keeps issue-intake as the lane-selection input. But issue-intake's `recommended_lane` is fixed to `issue-to-pr | work-plan | reply-only | manual-review` (SKILL.md:57; X.yaml fixtures at lines 273, 286, 317). There is no documented routing from `intake_report` to `support-response`, no graph/runner that calls it, and no statement that `support-response` is the executor for `recommended_lane=reply-only`. As written, the new lane has no caller.
  - Recommendation: Pick one explicitly in this spec and lock it with a Phase 3 AC: (a) extend the `recommended_lane` enum to include `support-response`, scope the intake SKILL.md/X.yaml edit, add intake fixtures, and add a contract-test AC that fails if the enum lacks `support-response`; or (b) declare in Decisions that `support-response` is the canonical executor for `recommended_lane=reply-only` (and for ask-for-info/manual-review escalations from there), name the orchestrator that performs that dispatch, and add a Phase 3 AC grepping for the wiring point. Without one of these, Phase 3 cannot be implemented coherently.
  - Question: Is support-response routed by widening intake's recommended_lane enum, or by declaring it the executor for `recommended_lane=reply-only` invoked from an upstream graph?
  - Recommended answer: Declare it the executor for `reply-only` (and a routing-target for `manual-review` when the cause is support context, not policy) so intake's enum and existing X.yaml fixtures stay stable. Phase 3 then adds the dispatcher pointer and a fixture asserting reply-only → support-response handoff.
- [medium/blocks approval] `harden-r2-2` scope - Touchpoints still list edits to issue-intake and issue-triage SKILL.md + X.yaml despite the sibling-only decision.
  - Status: open
  - Grounded in: spec_gap:touchpoints vs decisions
  - Evidence: Decisions (lines 132-138) commits to a sibling skill and says intake/triage `remains` unchanged in role. Touchpoints (lines 144-147) still list `skills/issue-intake/SKILL.md`, `skills/issue-intake/X.yaml`, `skills/issue-triage/SKILL.md`, `skills/issue-triage/X.yaml`. There is no acceptance gate bounding the size or nature of edits to those trees, so an implementer can legitimately reshape intake/triage prompts and call it in-scope.
  - Recommendation: Either (a) move issue-intake/issue-triage paths to Out of Scope unless harden-r2-1 lands as 'widen the enum' — in which case keep them in Touchpoints but add an AC that the intake X.yaml diff is limited to enum + one routing fixture; or (b) replace the four bullets with an explicit single-line entry like `skills/issue-intake/SKILL.md (routing pointer only)` and add a Phase 3 AC that asserts the intake prompt's structural sections are unchanged. The current state is incoherent with the Decisions block.
  - Question: Should issue-intake/issue-triage touchpoints be moved to Out of Scope, or kept with a bounded 'routing pointer only' clause + AC?
  - Recommended answer: Kept with a bounded clause: 'issue-intake SKILL.md/X.yaml may only add the routing pointer/example to support-response; no other change.' Same for issue-triage. Add a p3 grep AC that fails if intake/triage prompts gain new sections.
- [medium/blocks approval] `harden-r2-3` acceptance - p4_ac2's `-- support_response` filter matches zero tests until `mod support_response;` is added to integration.rs, so the gate is structurally inert.
  - Status: open
  - Grounded in: code:oss/crates/runx-runtime/tests/integration.rs:8-50, code:oss/package.json:53
  - Evidence: integration.rs is a single-binary aggregator listing each test as `mod <name>;` (lines 8-50). It contains `mod support;` (a shared helper module, line 47) but no `mod support_response;`. `cargo test ... --test integration -- support_response` exits 0 with `0 tests run` when nothing matches. p2_ac1's fixture-presence precondition prevents the empty-fixture pass but does not prevent the missing-module pass.
  - Recommendation: Add a second structural precondition: `test -f crates/runx-runtime/tests/support_response.rs && grep -q '^mod support_response;' crates/runx-runtime/tests/integration.rs && cargo test ...`. Or include a positive-count assertion by piping cargo's output through `tee` + grep for `test result:.*[1-9]` — pick whichever matches the convention in scripts/check-integration-test-modules.mjs (which the repo already runs).
  - Question: Should p4_ac2 add a `mod support_response;` presence check, or is binding to the existing `tests:integration-modules:check` script enough?
  - Recommended answer: Add the explicit `grep -q '^mod support_response;' crates/runx-runtime/tests/integration.rs` precondition. It is cheap, local to this spec, and self-documenting; the module-check script can stay as the workspace-wide invariant.
- [low/advisory] `harden-r2-4` acceptance - p2_ac1 alternation under-specifies the eight declared fixture categories.
  - Status: open
  - Grounded in: spec_gap:phase2 p2_ac1 vs phase2.changes
  - Evidence: Phase 2 enumerates eight fixture cases (answer-now, ask-for-info, duplicate, concrete-bug → create-issue/build-fix, docs/onboarding gap, account/billing manual review, unsafe customer copy, low-confidence/no-action). p2_ac1 only requires one match across the alternation `reply_draft|ask_for_info|manual_review|build_fix|duplicate`. Fixtures could ship with only `ask_for_info` cases and the gate passes.
  - Recommendation: Split into per-category presence checks (one rg per category) or assert directory cardinality, e.g. `[ $(ls fixtures/operational-intelligence/support-response | wc -l) -ge 8 ]` plus the existing alternation. Either form makes Phase 2 a credible deterministic gate.
- [low/advisory] `harden-r2-5` acceptance - Phase 4's 'fail closed on missing thread locator' invariant has no acceptance check.
  - Status: open
  - Grounded in: spec_gap:phase4.changes 'fails closed when policy requires reply-in-thread'
  - Evidence: Phase 4 Changes (line 293) says 'Ensure missing thread locator fails closed when policy requires reply-in-thread.' Phase 4's only ACs are p4_ac1 (story vocabulary grep) and p4_ac2 (fixture+cargo). Nothing asserts the policy-gated fail-closed behavior — neither a negative fixture nor a contract test.
  - Recommendation: Add a Phase 2 fixture for `policy_requires_reply_in_thread + missing thread_locator` whose expected output is `manual_review` (no reply_draft) and a Phase 4 AC: `rg -n 'thread_locator.*null|missing_thread_locator' fixtures/operational-intelligence/support-response` AND a vitest/cargo assertion that this fixture produces `manual_review`.

### round-3

Status: in_progress
Started: 2026-05-27T16:09:11Z
Ended: none

Checks:
- none

Issues:
- none


## Planning Log

- Split from the parent operational-intelligence program spec.
- Existing `issue-intake`, `issue-triage`, `issue-to-pr`, and `work-plan`
  skills should be reused rather than duplicated.
- Live support provider integrations and Nitrosend Slack UX are deferred to the
  Nitrosend integration child.
