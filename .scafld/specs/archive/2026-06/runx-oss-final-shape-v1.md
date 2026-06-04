---
spec_version: '2.0'
task_id: runx-oss-final-shape-v1
created: '2026-06-04T00:00:00Z'
updated: '2026-06-04T00:00:00Z'
status: completed
harden_status: passed
size: large
risk_level: high
---

# runx final shape: clean core and product lanes

## Current State

Status: completed
Current phase: archived
Next: none
Reason: Executed as the 2026-06 hard final-shape cleanup. Kept here only as
historical implementation context.
Blockers: none.
Allowed follow-up command: `scafld handoff runx-oss-final-shape-v1`
Latest runner update: 2026-06-04T00:00:00Z
Review gate: not_started

## Summary

This spec lands the ultimate final shape for runx after the hard cutover:

- The Rust/kernel crates execute governed graphs, seal receipts, parse/validate
  contracts, and host generic effects. They do not own GitHub provider workflows,
  target repo PR orchestration, or post-merge closure.
- The OSS core skill lane for code changes is the existing TS tool graph:
  `issue-intake` routes, `work-plan` composes, `issue-to-pr` runs scafld,
  `outbox.build_pull_request` packages the draft PR, and `thread.push_outbox`
  performs the provider mutation through the thread adapter.
- Product closure/advance behavior stays in product orchestration. Nitrosend
  already owns that lane in GitHub Actions, Node scripts, and Rails outcome
  callbacks.
- Dead package/tool/runtime surfaces are deleted outright. No shims, no deprecated
  exports, no compatibility names.
- CI gains guards that make this final shape enforceable instead of relying on
  reviews to notice accumulated debris.

It also scopes the remaining non-GitHub debris: payment/finality naming and
reserved state, cloud payment-finality worker/config gaps, stale payment-rails
docs, package/version drift, fixture-only artifacts, duplicate specs, and CI
coverage holes.

## Final Shape Decision

### Keep

- `skills/issue-intake`, `skills/work-plan`, and `skills/issue-to-pr`.
- `tools/outbox/build_pull_request`, `tools/thread/push_outbox`, and the GitHub
  adapter used by `thread.push_outbox`.
- Provider-neutral contracts that the skill graph actually uses:
  `runx.receipt.v1`, reference/act/decision schemas, `runx.operational_policy.v1`,
  thread/outbox provider packet shapes, runner manifest ABI, and effect finality.
- Nitrosend's product GitHub lane:
  workflow dispatch, merged PR/deployment verification, source issue close/reopen,
  Slack update, outcome callback ledger, and reconciler.

### Cut

- `packages/core/**`, including committed `dist/**`; the package has no active
  OSS or cloud importer and must not remain as a build-only package.
- Rust `post_merge_observer` runtime and contracts.
- Rust `target_runner` runtime and contracts, unless a production caller is added
  before deletion. The current live PR path is TS `thread.push_outbox`.
- Orphan tools with no production skill consumer: `fs.delete`, `fs.write_json`,
  and `thread.handoff_state`.
- Duplicate/stale specs, empty directories, fixture-only modules, old docs, and
  committed build outputs.

### Rename

- The final provider-state concept is not `post_merge_observer`. Use
  `provider_outcome` in runx OSS and `engineering outcome` in Nitrosend product
  docs/code where the Rails callback is product-specific.
- `post_merge_observation` metadata in the PR outbox packet becomes
  `provider_outcome_observation`. This is a SERIALIZED field of
  `runx.outbox-entry.pull-request.v1` (written at
  `build_pull_request/src/index.ts:251`, carried at `push_outbox/src/index.ts:467`);
  rename it atomically across the writer, the carrier, and the three tool fixtures
  in one commit. Verified safe: no Nitrosend, cloud `@runx/protocol`, GitHub
  adapter, or sealed-receipt path reads it (`github_adapter.mjs` already emits a
  DISTINCT `provider_outcome` field; do not conflate or dedup the two).
- DO NOT rename `target_runner_not_allowed`. Despite the name it is NOT a Rust
  `target_runner` API; it is a serialized finding `code` of the KEPT
  `runx.operational_policy.v1` contract, emitted by the runner-admission path in
  BOTH the Rust kernel (`crates/runx-contracts/src/operational_policy/evaluate.rs:280`)
  and the published `@runxhq/contracts` mirror
  (`packages/contracts/src/schemas/operational-policy.ts:604`), parity-tested and
  surfaced in CLI output. A non-additive rename breaks the mirror, the parity
  tests, and any operator branching on the code, contradicting "core skills keep
  working". A neutral name, if ever wanted, must be an additive read-alias across
  both sides plus all operational-policy fixtures.

## Evidence

- Nitrosend workflow runs on merged PR close and dispatch inputs carry deployment
  and verification metadata:
  `.github/workflows/issue-intake.yml:4`,
  `.github/workflows/issue-intake.yml:116`,
  `.github/workflows/issue-intake.yml:223`,
  `.github/workflows/issue-intake.yml:393`.
- Nitrosend merged PR closure short-circuits before runx skill execution:
  `scripts/issue-intake.mjs:117`, `scripts/issue-intake.mjs:137`,
  `scripts/issue-intake.mjs:4136`, `scripts/issue-intake.mjs:4195`.
- Nitrosend target-repo closure dispatch is Node/GitHub API:
  `scripts/runx-target-closure.mjs:184`, `scripts/runx-target-closure.mjs:203`,
  `scripts/runx-target-closure.mjs:216`, `scripts/runx-target-closure.mjs:232`.
- Nitrosend source mutation and ledger are product-owned:
  `scripts/post-issue-intake-comments.mjs:170`,
  `scripts/post-issue-outcome.mjs:43`,
  `api/app/services/issue_intake/outcome_callback.rb:1`,
  `api/app/services/issue_intake/reconciler.rb:46`.
- Core skill graph consumers:
  `skills/work-plan/X.yaml:247`, `skills/work-plan/X.yaml:257`,
  `skills/work-plan/X.yaml:267`, `skills/work-plan/X.yaml:277`,
  `skills/issue-intake/X.yaml:117`, `skills/issue-to-pr/X.yaml:603`,
  `skills/issue-to-pr/X.yaml:627`, `skills/issue-to-pr/X.yaml:655`.
- The skill's own contract says provider PR mutation belongs at
  `thread.push_outbox`, not in Rust:
  `skills/issue-to-pr/SKILL.md:21`, `skills/issue-to-pr/SKILL.md:25`.
- The live TS provider mutation surface is `thread.push_outbox` and
  `github_adapter.mjs`:
  `tools/thread/push_outbox/src/index.ts:43`,
  `tools/thread/push_outbox/src/index.ts:117`,
  `tools/thread/github_adapter.mjs:508`,
  `tools/thread/github_adapter.mjs:542`,
  `tools/thread/github_adapter.mjs:616`,
  `tools/thread/github_adapter.mjs:692`.
- Rust `post_merge_observer` and `target_runner` are public runtime modules but
  no production skill graph uses them:
  `crates/runx-runtime/src/lib.rs:38`,
  `crates/runx-runtime/src/lib.rs:58`,
  `crates/runx-runtime/src/post_merge_observer.rs:255`,
  `crates/runx-runtime/src/execution/target_runner.rs:302`.
- Orphan tools exist only as manifests/fixtures/tests:
  `tools/fs/delete/manifest.json:3`,
  `tools/fs/write_json/manifest.json:3`,
  `tools/thread/handoff_state/manifest.json:3`,
  `tests/thread-handoff-state-tool.test.ts:8`.
- Payment/effect residuals:
  `crates/runx-pay/src/runtime.rs:43` names a generic `EffectSupervisor` while
  `runtime.rs:107` and `runtime.rs:460` immediately interpret the payload as
  payment rail evidence; `crates/runx-pay/src/supervisor.rs:19` carries the
  payment evidence type; `supervisor.rs:32` and `supervisor.rs:34` carry
  future-looking optional fields.
- Payment finality store residuals:
  `crates/runx-pay/src/state.rs:120`, `state.rs:134`, `state.rs:432`, and
  `state.rs:483` define finality records/events used by tests but not a live
  rail worker path.
- Cloud payment finality residuals:
  `cloud/packages/billing/src/payment-settlements.ts:17` and `:59` still expose
  payment-settlement reconciliation; `cloud/package.json:32`,
  `cloud/packages/billing/package.json:23`, and
  `cloud/scripts/reconcile-payment-settlements.ts:5` preserve settlement naming.
- Cloud worker/config gap:
  `cloud/packages/api/src/server-config.ts:102`, `:200`, and `:303` parse a
  payment-finality worker flag; `cloud/packages/worker/src/payment-finality-worker.ts:29`
  provides a worker factory; `cloud/packages/api/src/worker-server.ts:33` starts
  metering but not payment finality.
- Cloud deploy/docs gap:
  `cloud/deploy/.env.example:17` lists finality env vars, while
  `cloud/deploy/prod.manifest.yaml:80` and `:91` omit them from required/passthrough;
  `cloud/docs/payments.md:120`, `:192`, and `:208` still use settlement/worker
  language that does not match the runtime.
- Config/package debris:
  `oss/tsconfig.typecheck.json:10`, `oss/tsconfig.typecheck.json:11`,
  `oss/tsconfig.runtime.json:9`, and `oss/vitest.config.ts:10` still include
  old app/plugin roots; `oss/package.json:78`,
  `cloud/packages/db/package.json:26`, and `cloud/packages/auth/package.json:39`
  pin old `@runxhq/contracts` workspace ranges despite `packages/contracts/package.json:3`
  being `0.3.0`; `crates/runx-core/Cargo.toml:29` and
  `crates/runx-pay/Cargo.toml:34` carry inert `default = ["std"]` features.
- Stale plans/specs:
  `plans/payment-rails.md:324`, `:377`, and `:393` still mention
  `effect_settlement_receipt` / `EffectSettlementReceipt`; active and draft
  `registry-hosted-cutover-v1.md` share the same task id.
- Root CI gap:
  root `package.json:20` defines `verify:fast`, but root
  `.github/workflows/publish-runx-skills.yml:70` only builds OSS packages; root
  CI does not enforce `pnpm verify:fast` on PR/push.

## Objectives

- Preserve every shipped core skill behavior while removing every unused runtime,
  package, fixture, and tool surface.
- Make the ownership boundary explicit:
  - Rust kernel: graph execution, contracts, receipts, generic effects.
  - OSS TS tools: source-thread/outbox provider mutation.
  - Nitrosend: product GitHub closure/advance and outcome ledger.
  - Cloud: hosted source ingress/admission/protocol, not target PR execution.
- Replace post-merge/Rust-runner terminology with provider-outcome terminology.
- Resolve payment/finality limbo: no generic-looking payment-only abstraction, no
  production finality state that is written only by tests, and no live cloud flag
  that does not start a worker.
- Wire CI so dead modules, orphan tools, committed dist, duplicate specs, and
  advisory-only dep/API drift cannot pass.

## Invariants

- **No compatibility layers.** Do not keep old exports, aliases, deprecated fields,
  or parallel names for deleted surfaces.
- **Core skills keep working.** All 51 official skills remain locked; `issue-intake`
  still recommends `issue-to-pr`; `work-plan` still composes `issue-to-pr`;
  `issue-to-pr` still reaches `outbox.build_pull_request` and both
  `thread.push_outbox` steps.
- **The provider mutation boundary stays TS.** No in-kernel GitHub, Slack, or
  target-repo provider client returns.
- **Nitrosend closure stays product-owned.** Merged PR, deployment verification,
  source issue close, Slack update, callback ledger, and reconciler behavior must
  keep passing in Nitrosend.
- **Effect/finality naming stays current.** `EffectFinalityReceipt` is the target
  generic receipt name. NOTE: at HEAD the LIVE name is still
  `runx.effect_settlement_receipt.v1` (a serialized `#[runx_schema]` id at
  `crates/runx-contracts/src/receipt.rs:30/43/59` emitting the committed artifact
  `schemas/effect-settlement-receipt.schema.json`). It is not yet renamed, so this
  is a serialized schema-id change that Phase 4 must perform atomically (const +
  `#[runx_schema]` id + artifact filename + schema_validation/wire fixtures); the
  invariant scans (p9_ac4) and docs-only scans only pass after that rename lands.
  Cloud already expects `runx.effect_finality_receipt.v1` (inert today), so the
  rename aligns the wire rather than breaking a live consumer.
- **No live flag without a live process.** A cloud env var or readiness gate may
  exist only if the runtime starts the corresponding process or the feature is
  explicitly marked non-prod and not required.
- **No hidden package zombies.** `@runxhq/core` must not exist, be imported, be
  exported, be built, or appear in publish checks except archived docs/specs.
- **Digest-bound evidence contract is frozen.** `PaymentSupervisorSettlementEvidence`
  (`crates/runx-pay/src/supervisor.rs`) is a `#[serde(deny_unknown_fields)]` struct
  folded into the sealed-receipt `supervisor_evidence_digest` and mirrored by the
  cloud Stripe executor. Its fields (incl. `shared_payment_token_ref`,
  `admission_token_digest`) cannot be removed or renamed without breaking cloud +
  sealed-receipt re-verification. Changes here are a rail-drop product decision, not
  cleanup.
- **No fixture-only production exports** EXCEPT reserved, fixture-locked contract
  verifiers that the schema/parity surface depends on. A `src/` module must have a
  real runtime/CLI/skill/cloud caller, OR be an explicitly-listed reserved verifier
  (e.g. `crates/runx-pay/src/refunds.rs` `admit_refund`/`verify_refund_admission_case`,
  fixture-driven via `fixtures/effect-finality/refund-admission`, kept ahead of the
  out-of-scope real-rail build). Anything else under `src/` is deleted.

## Scope

In scope:

- OSS package deletion: `packages/core/**`, workspace manifests, build aliases,
  publish checks, boundary checks, docs, and stale tests.
- OSS Rust hard cut: `post_merge_observer` and `target_runner` runtime/contracts,
  related tests, fixture-only receipts, and stale docs.
- OSS skill/tool cleanup: orphan tools, manifests, fixtures, tests, and lock/catalog
  regeneration.
- OSS naming hard cut: `post_merge_observation` ->
  `provider_outcome_observation` (serialized outbox-packet field; atomic across
  writer + carrier + fixtures). `target_runner_not_allowed` is NOT renamed (kept
  `operational_policy.v1` finding code; see Rename).
- Payment/effect cleanup inside `runx-pay`: honest payment-local supervisor
  naming or truly generic non-payment proof; no unused optional evidence fields;
  no finality record/event store without a live caller.
- Cloud payment-finality cleanup: settlement naming, worker startup/flag, deploy
  manifest, docs, and package exports.
- Cross-workspace dependency/config cleanup: `@runxhq/contracts` ranges,
  inert Cargo features, stale TS roots, duplicate specs, stale fixtures.
- CI hardening for final-shape enforcement.
- Nitrosend validation only; do not move Nitrosend code into runx.

Out of scope:

- Real payment rails beyond existing effect finality shape.
- Cloud-to-kernel bridge.
- OpenAPI front.
- A new runx-owned GitHub closure service.
- Auto-merge. The human merge gate remains.

## Risks

- **Breaking the core skill lane.** `issue-to-pr` is a real official skill, and
  Nitrosend depends on it for initial issue-to-PR work. Mitigation: lock graph
  edges before deleting Rust surfaces; run parser, harness, dogfood, and
  Nitrosend wrapper tests.
- **Confusing closure ownership.** Nitrosend needs post-merge close/advance, but
  runx Rust does not. Mitigation: codify product ownership and keep only
  provider-outcome metadata in the TS outbox lane.
- **Deleting/renaming a serialized contract shape.** The tree is NOT greenfield for
  serialized surfaces: sealed-receipt fields, schema ids, the operational-policy
  finding codes, the outbox packet fields, and the Stripe evidence digest are
  consumed by fixtures, the `@runxhq/contracts` mirror, the cloud `@runx/protocol`
  fork, and Nitrosend. Mitigation: per Invariants, do not rename/remove a serialized
  name as a "free hard cut"; either keep it, or rename it ATOMICALLY across every
  mirror + fixture in one commit, regenerating canonical oracles with
  `git diff --exit-code`.
- **Gate inflation.** `verify:fast` must remain fast enough to use locally.
  Mitigation: split fast-vs-heavy only when both are required in CI; add plan
  checks so unwired tests cannot silently accumulate.
- **Prematurely deleting real future-rail work.** Some finality records are useful
  for real rails, but no current process writes them. Mitigation: either wire a
  production caller in this spec or delete them and let the real-rails spec add
  them back when it owns the runtime path.

## Phase 0: Confirm Ownership and Lock Skill Graph

Objective: make the live graph inventory machine-checkable before deletion.

Changes:

- Add a skill/tool graph guard that scans `skills/**/X.yaml` and official locks:
  - `issue-to-pr` must use `outbox.build_pull_request` and `thread.push_outbox`.
  - `work-plan` may invoke `issue-to-pr`.
  - `issue-intake` may recommend `issue-to-pr`.
  - no production skill may use `fs.delete`, `fs.write_json`,
    `thread.handoff_state`, `target_runner`, or `post_merge_observer`.
- Add a guard that fails when any tool manifest exists with zero production skill
  consumers unless it is explicitly allowlisted with an owner and expiry.
- Add a guard that fails on source references to Rust `target_runner` or
  `post_merge_observer` outside archived specs before the cut phase starts.

Acceptance:

- [ ] `p0_ac1` command - current skill graph inventory is explicit
  - Command: `rg -n "outbox\\.build_pull_request|thread\\.push_outbox|fs\\.delete|fs\\.write_json|thread\\.handoff_state|target_runner|post_merge_observer" skills tools packages/cli/src packages/contracts/src`
  - Expected kind: `reviewed_output`
- [ ] `p0_ac2` command - official skill lock remains complete
  - Command: `node -e "const fs=require('fs');const lock=JSON.parse(fs.readFileSync('packages/cli/src/official-skills.lock.json','utf8'));if(lock.length!==51)process.exit(1)"`
  - Expected kind: `exit_code_zero`

## Phase 1: Delete @runxhq/core

Objective: remove the largest dead package instead of keeping a private build-only
surface.

Changes:

- Delete `packages/core/**`, including committed `dist/**`.
- Remove EVERY hardcoded `@runxhq/core`/`core` reference (Phase 1 understated this):
  `scripts/check-boundaries.mjs`, `scripts/check-receipt-importers.ts:331`,
  `scripts/check-rust-cli-release-artifacts.ts:378` (forbidden-dep array),
  `scripts/test-boundaries.mjs:70`, `docs/contract-schema-consumer-inventory.md:18`
  (lists bare `core` as a live consumer), plus workspace manifests, tsconfig/vitest
  aliases, publish/release checks, docs, stale fixtures, and tests.
- Resolve the maturity parity pair: `packages/core/src/registry/maturity.ts`
  (`computeMaturity`) is the TS side of `crates/runx-core/tests/maturity_parity.rs`
  (both read `fixtures/kernel/maturity/compute-maturity-cases.json`). Move
  `computeMaturity` to a kept package OR explicitly accept the Rust+fixture pair as
  the remaining guard; do not silently drop cross-language maturity coverage.
- Regenerate `pnpm-lock.yaml` in the SAME commit (deleting `packages/core` removes
  its lock entry); Nitrosend CI runs `pnpm install --frozen-lockfile` against a
  pinned `RUNX_REF`, so a stale lockfile breaks its build.
- Update `scripts/check-boundaries.mjs` from "core package must not expose old
  domains" to "core package must not exist".
- Add guards:
  - no `packages/core`;
  - no `@runxhq/core` import or package dependency in OSS or cloud source;
  - no committed package `dist/**`.

Acceptance:

- [ ] `p1_ac1` command - package deleted
  - Command: `test ! -e packages/core`
  - Expected kind: `exit_code_zero`
- [ ] `p1_ac2` command - no live core import/dependency remains
  - Command: `! rg -n "@runxhq/core|packages/core" packages crates scripts tests ../cloud/packages ../cloud/scripts --glob '!**/node_modules/**' --glob '!**/dist/**' --glob '!**/target/**'`
  - Expected kind: `exit_code_zero`
- [ ] `p1_ac3` command - boundary guard enforces deletion
  - Command: `pnpm boundary:check && pnpm test:boundary`
  - Expected kind: `exit_code_zero`
- [ ] `p1_ac4` command - lockfile consistent (Nitrosend's frozen install passes)
  - Command: `pnpm install --frozen-lockfile`
  - Expected kind: `exit_code_zero`

## Phase 2: Hard-Cut Rust Provider Lanes

Objective: remove Rust provider/target orchestration that has no production
skill caller.

Changes:

- Delete `crates/runx-runtime/src/post_merge_observer.rs` and
  `crates/runx-runtime/src/post_merge_observer/**`.
- Delete the Rust `post_merge_observer` contracts module and tests, and remove its
  re-export block (`crates/runx-contracts/src/lib.rs:185-194`, ~14
  `PostMergeObserver*` types) AND the runtime re-export (`runtime/lib.rs:33`
  `pub mod post_merge_observer;`).
- Delete `crates/runx-runtime/src/execution/target_runner.rs` and
  `crates/runx-runtime/src/execution/target_runner/**`.
- Delete the Rust `target_runner` contracts module and remove its re-export block
  (`crates/runx-contracts/src/lib.rs:219-230`, ~20 `TargetRepoRunner*` types) AND
  the runtime re-export (`lib.rs:47` `pub mod target_runner;`). Scope deletions to
  the `target_runner` MODULE / `TargetRepoRunner*` types ONLY; never touch the kept
  `operational_policy` finding code `target_runner_not_allowed`.
- Pre-acknowledge the `cargo public-api` breaking diff from removing those ~34
  re-exported types (Phase 7's now-blocking public-api gate will flag them).
- Do NOT file-delete the post-merge canonical fixture
  `fixtures/contracts/canonical-json/.../post-merge-observer-merged-verified.json`
  or its siblings: it is ALSO a kept `runx.receipt.v1` c14n-oracle case (one of
  three) and a Nitrosend dogfood fixture. Instead: (a) regenerate the harness-spine
  + c14n oracle via `crates/runx-receipts/examples/generate_harness_spine_fixtures.rs`
  so the third oracle case becomes a generic kept-receipt case, gated with
  `git diff --exit-code` so no stale case remains; (b) rewrite
  `crates/runx-contracts/tests/nitrosend_external_fixture.rs` to derive its target
  plan WITHOUT `plan_target_repo_runner` (or via the kept `operational_policy`
  admission path); (c) delete `packages/contracts/src/schemas/post-merge-observer-fixture.test.ts`
  (a reader of the deleted fixture, present in BOTH vitest lanes) and grep
  `packages/contracts` + `crates/**/tests` for any other deleted-fixture reader.
- Rename the SERIALIZED TS outbox-packet field `post_merge_observation` ->
  `provider_outcome_observation` atomically across writer + carrier + the three
  fixtures (see Rename). Do NOT rename `target_runner_not_allowed`.
- Keep `runx.operational_policy.v1` actions such as `issue-to-pr`,
  `pr-fix-up`, and `merge-assist`; those are provider-neutral policy actions,
  not Rust target-runner APIs.

Acceptance:

- [ ] `p2_ac1` command - Rust provider lanes deleted (kept operational-policy code untouched)
  - Command: `! rg -n "post_merge_observer|PostMergeObserver|\\bmod target_runner\\b|TargetRepoRunner" crates/runx-runtime/src crates/runx-contracts/src`
  - Expected kind: `exit_code_zero`
- [ ] `p2_ac2` command - the serialized outbox field is renamed; the kept policy code stays
  - Command: `! rg -n "post_merge_observation" tools packages/contracts/src skills`
  - Expected kind: `exit_code_zero`
- [ ] `p2_ac3` command - kept Rust contract/receipt + TS graph suites still pass
  - Command: `pnpm exec vitest run tests/scafld-issue-to-pr-parser.test.ts tests/outbox-build-pull-request-tool.test.ts tests/thread-push-outbox-tool.test.ts && cargo test --manifest-path crates/Cargo.toml -p runx-contracts && cargo test --manifest-path crates/Cargo.toml -p runx-receipts`
  - Expected kind: `exit_code_zero`

## Phase 3: Delete Orphan Tools and Fixtures

Objective: only shipped skill surfaces remain in the tool catalog.

Changes:

- Delete `tools/fs/delete/**`, `tools/fs/write_json/**`, and
  `tools/thread/handoff_state/**`. Keep `fs.read`, `fs.write`, `fs.write_bundle`
  (only `fs.delete` + `fs.write_json` are cut).
- CARVE-OUT: deleting the `thread.handoff_state` TOOL must NOT touch the runx
  handoff CONTRACT (a separate, kept, serialized surface sharing the name token):
  keep `crates/runx-contracts/src/handoff.rs`, `schemas/handoff-state.schema.json`,
  `packages/contracts/src/schemas/handoff.ts`, and the published
  `content.handoff.v1` schema.
- Delete tests and fixture snapshots that existed only to keep those orphan tools
  alive.
- Regenerate/check official tool catalog artifacts.
- Add a no-orphan-tool guard to `verify:fast`.

Acceptance:

- [ ] `p3_ac1` command - orphan tools absent
  - Command: `test ! -e tools/fs/delete && test ! -e tools/fs/write_json && test ! -e tools/thread/handoff_state`
  - Expected kind: `exit_code_zero`
- [ ] `p3_ac2` command - no production graph references deleted tools
  - Command: `! rg -n "fs\\.delete|fs\\.write_json|thread\\.handoff_state" skills tools packages tests`
  - Expected kind: `exit_code_zero`
- [ ] `p3_ac3` command - tool catalog guard passes
  - Command: `pnpm verify:fast:plan-check && pnpm verify:fast`
  - Expected kind: `exit_code_zero`

## Phase 4: Payment and Effect Finality Residuals

Objective: remove limbo from the payment/effect layer. Nothing in production
should look generic while being payment-only, and nothing should be reserved
ahead of a real rail process without a caller.

Changes:

- Resolve the `EffectSupervisor` half-abstraction in `runx-pay`.
  - Preferred final shape: rename the trait and request/evidence types to
    payment-local names such as `PaymentEffectSupervisor`,
    `PaymentEffectSupervisorRequest`, `PaymentEffectSupervisorEvidence`, and
    `DeterministicPaymentEffectSupervisor`; remove the redundant `family` field
    from the payment-local request/evidence path because the crate is already
    payment-owned.
  - Alternative only if a real non-payment caller is added in this spec: move a
    genuinely generic supervisor trait to the generic runtime seam and prove it
    with a second non-payment effect family fixture.
- Do NOT cut the `PaymentSupervisorSettlementEvidence` fields
  `shared_payment_token_ref` / `admission_token_digest` (review correction). They
  are NOT unused: `admission_token_digest` is a LIVE serialized cross-boundary
  field consumed by `cloud/packages/stripe-executor`, and both are folded into the
  `#[serde(deny_unknown_fields)]` `supervisor_evidence_digest` bound into the
  sealed payment receipt; the Stripe-SPT rail (`stripe_spt.rs`) populates them with
  real values. Removing them is a breaking contract change that drops the SPT rail
  and breaks sealed-receipt re-verification + the cloud executor. Keep both.
  (Dropping the Stripe-SPT rail is a SEPARATE product decision; if taken, cut the
  fields from `runx-pay` AND `cloud/packages/stripe-executor` in the same change,
  delete/rewrite `stripe_spt.rs`, and regenerate every Some-valued evidence
  fixture. Note: keep `payment_admission_token_digest()` / `payment_admission.rs` —
  CLI-live, unrelated to the evidence field.)
- Rename the live `EffectSettlementReceipt` contract to `EffectFinalityReceipt`
  (and `EffectSettlementPhase` -> `EffectFinalityPhase`) atomically: the Rust const
  + `#[runx_schema]` id `runx.effect_settlement_receipt.v1` ->
  `runx.effect_finality_receipt.v1` (`receipt.rs:30/43/59`), the committed artifact
  `schemas/effect-settlement-receipt.schema.json` ->
  `effect-finality-receipt.schema.json` (`schema_artifacts.rs`), and every
  schema_validation/wire-conformance/receipt fixture carrying the discriminator, in
  one commit. This aligns the wire with cloud's existing
  `runx.effect_finality_receipt.v1` expectation (inert today). Keep
  `crates/runx-pay/src/refunds.rs` (reserved fixture-locked verifier).
- Cut `EffectSettlementFinalityRecord`, `EffectSettlementEventRecord`, and their
  store methods (`record_settlement_finality`, `record_settlement_event`,
  `lookup_settlement_finality`, `lookup_settlement_event` at
  `crates/runx-pay/src/state.rs:120/134/438/448/476/489`) unless this phase wires a
  live worker that records them; verified test-only at HEAD. Do NOT cut
  `record_settlement_intent` (it has a live src caller).
- Delete or rename the orphan `scripts/settlement-finality.mjs`; if retained, it
  must be named `effect-finality` and wired to a real script/test entry.
- Update payment fixtures/tests after the hard cut; do not keep compatibility
  aliases for removed metadata fields or old type names.

Acceptance:

- [ ] `p4_ac1` command - no generic-looking payment-only supervisor remains
  - Command: `! rg -n "pub trait EffectSupervisor|EffectSupervisorRequest|EffectSupervisorEvidence|DeterministicEffectSupervisor" crates/runx-pay/src crates/runx-pay/tests`
  - Expected kind: `exit_code_zero`
- [ ] `p4_ac2` command - the digest-bound payment evidence contract is intact (fields kept)
  - Command: `rg -n "shared_payment_token_ref" crates/runx-pay/src/supervisor.rs && rg -n "\\badmission_token_digest\\b" crates/runx-pay/src/supervisor.rs && cargo test --manifest-path crates/Cargo.toml -p runx-pay --all-features`
  - Expected kind: `exit_code_zero`
- [ ] `p4_ac3` command - no production finality record store without a caller
  - Command: `! rg -n "EffectSettlementFinalityRecord|EffectSettlementEventRecord|record_settlement_finality|record_settlement_event|lookup_settlement_finality|lookup_settlement_event" crates/runx-pay/src`
  - Expected kind: `exit_code_zero`
- [ ] `p4_ac4` command - effect finality tests still pass
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-pay --all-features`
  - Expected kind: `exit_code_zero`

## Phase 5: Cloud Payment Finality and Deployment Shape

Objective: cloud payment finality either has a live process and production config,
or no production flag/readiness claim. Settlement naming is split from finality
naming.

Changes:

- SCOPE GUARD: the settlement->finality rename touches TS type/interface/export/
  file/command IDENTIFIERS ONLY. It MUST NOT change persisted DB table/column names
  or the serialized `kind: "payment"` discriminator value — `PaymentSettlementIntentRecord`
  is a pg-persisted, discriminator-keyed row, and renaming stored values breaks
  existing rows. Enumerate the real files: `settlement-intents.ts`,
  `payment-finality-repair.ts`, `metering/service.ts`, `billing` index re-exports
  (note `payment:reconcile-finality` + `scripts/reconcile-payment-finality.ts`
  already exist from a partial prior split).
- Rename or split `cloud/packages/billing/src/payment-settlements.ts`:
  - topup settlement stays settlement-specific;
  - payment recovery/finality becomes `payment-finality-reconcile` or equivalent.
- Rename cloud exports/scripts:
  - `@runx/billing/payment-settlements` export becomes finality-specific or
    topup-specific;
  - root command `payment:reconcile-settlements` becomes
    `payment:reconcile-finality` for payment finality, with any topup settlement
    command kept separately;
  - `scripts/reconcile-payment-settlements.ts` is renamed/split.
- Resolve `RUNX_PAYMENT_FINALITY_WORKER_ENABLED`:
  - Wire `createPaymentFinalityWorker` into the hosted worker startup path and add
    deploy manifest secrets/passthrough/tests; or
  - remove the env var, server-config field, readiness requirement, and docs until
    a real worker process exists.
- Align deploy manifests:
  - if finality webhooks/worker are prod features, add required/passthrough envs
    and tests;
  - if not, mark them non-prod and remove readiness gating.
- Update `cloud/docs/payments.md` to use rail/finality wording, not settlement
  families for consumer payment finality.

Acceptance:

- [ ] `p5_ac1` command - settlement naming no longer fronts payment finality
  - Command: `cd ../cloud && ! rg -n "payment-settlements|reconcile-payment-settlements|payment:reconcile-settlements|PaymentSettlement" packages scripts docs package.json`
  - Expected kind: `exit_code_zero`
- [ ] `p5_ac2` command - worker flag is either wired or absent
  - Command: `cd ../cloud && rg -n "paymentFinalityWorkerEnabled|RUNX_PAYMENT_FINALITY_WORKER_ENABLED|createPaymentFinalityWorker" packages deploy docs scripts`
  - Expected kind: `reviewed_output`
- [ ] `p5_ac3` command - cloud gates
  - Command: `pnpm --dir ../cloud typecheck:server && pnpm --dir ../cloud check:structure && pnpm --dir ../cloud test`
  - Expected kind: `exit_code_zero`

## Phase 6: Config, Fixtures, Specs, and Package Hygiene

Objective: remove non-code debris that makes the repository look larger or less
finished than the product is.

Changes:

- Remove stale TS roots from OSS configs:
  - `apps/**/*.ts` and `plugins/**/*.ts` in typecheck/runtime/vitest configs if
    the roots are empty or retired.
- Align `@runxhq/contracts` workspace ranges with the actual `0.3.0` package
  version across OSS and cloud, or use `workspace:*` consistently.
- Remove inert Cargo features from `runx-core` and `runx-pay` if nothing compiles
  with `no_std`; otherwise add real `std` gating. Do not keep empty `std = []`
  as decoration.
- Remove stale empty dirs that are not intentional fixtures:
  `examples/framework-adapters`, `packages/core/src/policy`, and
  `tools/scafld/capture_checks`.
- Classify fixture-only directories and delete those with no live test/importer:
  `fixtures/payment-admission`, `fixtures/upstream-binding`,
  `fixtures/quality/bad-artifacts.json`, `fixtures/runtime-semantics`, and the
  unused half of `fixtures/embedded-sdk-migration`. Keep only
  `runtime-service-boundary.json` if `runx-sdk` still consumes it.
- Delete or transition duplicate specs:
  keep the active `registry-hosted-cutover-v1.md`, remove the draft duplicate, and
  add a spec-lifecycle guard for duplicate task IDs and invalid active statuses.
- Update stale plans/docs:
  - `plans/payment-rails.md` and `plans/payment-rails-demo.md` to current
    `EffectFinalityReceipt`, generic effect-finality naming, and the actual
    post-cutover payment authority/proof homes;
  - `plans/codex-handoff-cutover-finalization.md` as superseded/completed or move
    it out of live handoff surfaces;
  - `docs/runtime-cutover-inventory.json` if it references deleted tests/files.

Acceptance:

- [ ] `p6_ac1` command - stale config roots gone
  - Command: `! rg -n "apps/\\*\\*/\\*\\.ts|plugins/\\*\\*/\\*\\.ts|plugins/\\*\\*|apps/\\*\\*" tsconfig*.json vitest*.ts`
  - Expected kind: `exit_code_zero`
- [ ] `p6_ac2` command - package ranges aligned
  - Command: `! rg -n '"@runxhq/contracts": "workspace:\\^0\\.2' package.json packages/*/package.json ../cloud/package.json ../cloud/packages/*/package.json`
  - Expected kind: `exit_code_zero`
- [ ] `p6_ac3` command - inert Cargo features gone or real
  - Command: `! rg -n 'default = \\["std"\\]|^std = \\[\\]' crates/runx-core/Cargo.toml crates/runx-pay/Cargo.toml`
  - Expected kind: `exit_code_zero`
- [ ] `p6_ac4` command - stale payment/finality docs gone
  - Command: `! rg -n "effect_settlement_receipt|EffectSettlementReceipt|ProofKind::PaymentRail|PaymentAuthorityBounds|PaymentCredentialForm" ../plans docs`
  - Expected kind: `exit_code_zero`
- [ ] `p6_ac5` command - no duplicate active/draft spec remains
  - Command: `node -e "const fs=require('fs'),p='.scafld/specs';const ids=new Map();for(const dir of ['active','drafts'])for(const f of fs.existsSync(p+'/'+dir)?fs.readdirSync(p+'/'+dir):[])if(f.endsWith('.md')){const s=fs.readFileSync(p+'/'+dir+'/'+f,'utf8');const m=s.match(/^task_id: (.+)$/m);if(m){const k=m[1].trim();if(ids.has(k))throw new Error('duplicate spec '+k);ids.set(k,dir+'/'+f)}}"`
  - Expected kind: `exit_code_zero`

## Phase 7: Gate Hardening

Objective: CI enforces the final shape across root, OSS, and cloud.

Changes:

- Add a root PR/push CI workflow that runs root `pnpm verify:fast`, so root
  cutover checks cover OSS and cloud together.
- Make `cargo-deny` and `cargo public-api` fail CI, not advisory-only. If tool
  installation is flaky, split tool install from the required check instead of
  using `continue-on-error`.
- Ensure every non-heavy `.test.ts` suite is either included in a required test
  lane or explicitly documented as heavy with its own required CI job.
- Extend root `scripts/check-contract-cutover.mjs` or add a sibling guard for
  live spec/doc lifecycle checks. Do not hide duplicate active/draft specs by
  ignoring `.scafld/specs`.
- Add guards for:
  - no `packages/core`;
  - no committed package/app `dist/**` except explicitly generated release/schema
    artifacts;
  - no empty leftover directories outside fixture allowlists;
  - no duplicate active/draft specs with identical task IDs;
  - no orphan tool manifests;
  - no Rust provider lane modules;
  - no stale settlement/finality names in live source/docs;
  - no duplicate runtime compatibility checks (dedup `checkNoRuntimeCompatModules`,
    currently duplicated across three gates);
  - a cross-language tool `source_hash` parity test (Rust `hash_tool_source` vs TS
    `hashToolSource` over a shared fixture covering escaped/mixed quotes) — the two
    must agree byte-for-byte or manifest hashes silently desync;
  - a content guard forbidding NEW in-kernel provider clients (`reqwest::Client`
    construction outside `runtime_http.rs`, or github/slack/target-repo client
    symbols in `runx-runtime`) — the existing deny.toml guard only restricts the
    reqwest dependency EDGE, not new call sites.
- Keep `cutover:legacy-check --final` as the domain-free Rust guard.

Acceptance:

- [ ] `p7_ac1` command - root gate runs
  - Command: `cd .. && pnpm verify:fast`
  - Expected kind: `exit_code_zero`
- [ ] `p7_ac2` command - OSS fast gate cannot skip live TS integration accidentally
  - Command: `pnpm verify:fast:plan-check && pnpm verify:fast`
  - Expected kind: `exit_code_zero`
- [ ] `p7_ac3` command - full TS suites have explicit required lanes
  - Command: `pnpm test && pnpm --dir ../cloud test`
  - Expected kind: `exit_code_zero`
- [ ] `p7_ac4` command - Rust dep/API drift is blocking
  - Command: `node scripts/check-rust-kernel-parity.mjs`
  - Expected kind: `exit_code_zero`
- [ ] `p7_ac5` command - parity/cargo-deny/cargo public-api are required, not advisory
  - Command: `rg -n -B1 -A3 "check-rust-kernel-parity|cargo-deny|cargo public-api|cargo-public-api" .github/workflows/ci.yml`
  - Expected kind: `reviewed_output` (none under `continue-on-error: true`)

## Phase 8: Cross-Repo Nitrosend Validation

Objective: prove the product GitHub lane still works after runx deletes the Rust
provider scaffolding and cleans provider-outcome naming.

Run from `/Users/kam/dev/nitrosend`. CRITICAL: the wrapper tests pass against the
OLD pinned `RUNX_REF` and so can go green WITHOUT exercising the new runx surface.
This phase must first bump `RUNX_REF` to the post-spec runx HEAD and run Nitrosend's
exact CI prelude against it, THEN run the wrapper tests. The `RUNX_REF` bump must
not precede the Phase 1/6 lockfile regen.

Acceptance:

- [ ] `p8_ac0` command - new runx surface builds + installs under Nitrosend's CI prelude
  - Command: `cd /Users/kam/dev/nitrosend && git -C runx fetch && git -C runx checkout <post-spec-runx-HEAD> && pnpm --dir runx install --frozen-lockfile && pnpm --dir runx build`
  - Expected kind: `exit_code_zero`
- [ ] `p8_ac1` command - Nitrosend wrapper tests (against the new runx build)
  - Command: `node --test scripts/source-loader.test.mjs scripts/triage.test.mjs scripts/issue-intake.test.mjs scripts/github-issue-thread.test.mjs scripts/post-issue-intake-comments.test.mjs scripts/post-issue-outcome.test.mjs scripts/runx-target-closure.test.mjs scripts/scafld-command-review.test.mjs scripts/runx-harness.test.mjs`
  - Expected kind: `exit_code_zero`
- [ ] `p8_ac2` manual - merged PR closure path still closes only after verified deployment
  - Expected kind: `manual`

## Phase 9: Final Gates and Docs

Objective: close the sweep with accurate docs and green gates.

Changes:

- Update `README.md`, `docs/ts-interop-boundary.md`,
  `docs/rust-kernel-architecture.md`, `docs/issue-to-pr.md`, cloud payment docs,
  and root plans that still imply Rust owns target PR/post-merge orchestration or
  that payment finality is still named settlement.
- Run full Rust, TS, fixture, cloud, Nitrosend, and license gates.

Acceptance:

- [ ] `p9_ac1` command - full OSS gate
  - Command: `pnpm verify:fast && pnpm test && pnpm fixtures:harness:check && cargo fmt --manifest-path crates/Cargo.toml --all --check && cargo clippy --manifest-path crates/Cargo.toml --workspace --all-targets --all-features -- -D warnings && cargo nextest run --manifest-path crates/Cargo.toml --workspace --all-features && cargo test --manifest-path crates/Cargo.toml --workspace --all-features --doc`
  - Expected kind: `exit_code_zero`
- [ ] `p9_ac2` command - full cloud gate
  - Command: `pnpm --dir ../cloud verify:fast && pnpm --dir ../cloud typecheck:server && pnpm --dir ../cloud check:structure`
  - Expected kind: `exit_code_zero`
- [ ] `p9_ac3` command - license boundary
  - Command: `node .scafld/scripts/check-license-edges.mjs --check manifest-complete && node .scafld/scripts/check-license-edges.mjs --check identifiers && cargo metadata --manifest-path crates/Cargo.toml --format-version 1 | node .scafld/scripts/check-license-edges.mjs --check edges`
  - Expected kind: `exit_code_zero`
- [ ] `p9_ac4` command - final shape source scans
  - Command: `! rg -n "@runxhq/core|post_merge_observer|PostMergeObserver|target_runner|TargetRepoRunner|post_merge_observation|target_runner_not_allowed|effect_settlement_receipt|EffectSettlementReceipt|payment-settlements|reconcile-payment-settlements" packages crates tools skills docs scripts ../cloud/packages ../cloud/scripts ../cloud/docs ../plans --glob '!**/archive/**'`
  - Expected kind: `exit_code_zero`

## Rollback

This is a hard cutover, not a compatibility migration.

- Phase 1 rollback restores `packages/core/**` only if a missed live importer is
  found. The preferred fix is to move that importer to the real owner, not to
  keep core.
- Phase 2 rollback restores Rust provider lanes only if Phase 0 missed a real
  production caller. Tests/fixtures alone are not a rollback reason.
- Phase 3 rollback restores an orphan tool only if a production skill uses it.
- Phase 4 rollback restores payment/finality fields or stores only if the same
  rollback restores a live caller that writes and validates them.
- Phase 5 rollback restores cloud finality env/worker names only as a complete
  feature slice: config, process startup, deployment manifest, readiness, docs,
  and tests. Do not restore a parsed-but-unused flag.
- Phase 6 rollback restores fixtures/config roots only when a live test/importer
  is restored with them.
- Phase 7 rollback must leave another required CI guard in place for the same
  invariant.
- Any rollback must also remove the final-shape guard that would otherwise fail
  the reverted surface; do not leave contradictory guards.

## Resulting Shape

- Rust OSS is the governed execution/contract/receipt/effect kernel.
- OSS TS tools own source-thread/outbox provider mutation for core skills.
- Nitrosend owns GitHub closure, deployment verification, Slack updates, and the
  Rails engineering-outcome ledger.
- Cloud owns hosted source ingress/admission/protocol and either runs a real
  payment-finality worker or does not expose a production worker flag.
- `@runxhq/core`, Rust `target_runner`, Rust `post_merge_observer`, orphan tools,
  generic-looking payment-only supervisor names, stale settlement/finality names,
  duplicate specs, fixture-only runtime exports, committed package/app build
  outputs, and advisory-only gates are gone.
