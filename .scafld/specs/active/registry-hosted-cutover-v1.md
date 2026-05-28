---
spec_version: '2.0'
task_id: registry-hosted-cutover-v1
created: '2026-05-28T00:00:00Z'
updated: '2026-05-28T00:00:00Z'
status: draft
harden_status: not_run
size: large
risk_level: high
---

# Registry hosted cutover

## Current State

Status: active
Current phase: phase4
Next: reconcile
Reason: Phase 1 (CLI -> Rust install client) merged as 5a64917. Phase 3 reduced to
deleting the orphaned `core/marketplaces` (merged 66755fb); `core/registry` is NOT
deleted because the hosted cloud imports it in ~24 source files (it IS the cloud's
registry-service library). core/registry is reclassified as that shared lib;
full deletion would require internalizing it into a cloud package and is deferred.
Blockers: full core/registry deletion blocked by the cloud dependency (out of
scope here); revives the failed `rust-ts-sunset-registry` (coordinate).
Allowed follow-up command: `scafld handoff registry-hosted-cutover-v1`
Latest runner update: 2026-05-28T00:00:00Z
Review gate: not_started

## Summary

Retire the TypeScript registry domain (`packages/core/src/registry`, ~2.6k LOC)
and `packages/core/src/marketplaces` (~280 LOC) by making the boundary explicit:
the install **client** is the Rust runtime, the registry **index/service** is the
hosted cloud, and the OSS TypeScript registry is the redundant middle that gets
deleted.

This is mostly subtraction. Both ends already exist:

- Rust (`crates/runx-runtime/src/registry/{refs,local,http,install,trust_anchor}.rs`)
  parses refs, ingests/stores locally, installs with signed-manifest verification,
  and its HTTP client already targets the cloud's `/v1/skills`,
  `/v1/skills/{owner}/{name}`, and `/v1/skills/{owner}/{name}/acquire`.
- Cloud serves exactly those endpoints (public, cache-`public`), backs them with a
  file store (`.data/runx-registry/`), computes trust/maturity at publish, gates
  publish behind an admin token, and exposes MCP discovery tools.

The earlier `rust-ts-sunset-registry` failed because it tried to port the whole TS
domain into Rust. The reframe that unblocks it: the service logic already lives in
cloud and the client already lives in Rust, so the TS is deleted, not moved.

## Objectives

- The hosted cloud is the single registry index/service (search, trust/maturity,
  publish, namespaces). No registry service logic remains in OSS TypeScript.
- The Rust runtime is the single install client. `runx install <ref>` resolves,
  fetches, verifies, and installs with no TypeScript on the path.
- `packages/core/src/registry` and `packages/core/src/marketplaces` are removed,
  leaving only shared types still consumed by presentation.
- OSS users install with no account: anonymous cloud read/acquire, plus
  `github:`/`file:`/url refs handled entirely client-side in Rust.

## Invariant (adoption line)

The cloud `/v1/skills` read and `/acquire` paths MUST stay anonymous, and the
`github:`/`file:`/url install refs MUST work client-side with no account or cloud
call. The hosted index is the better, optional source; never the only one. A
change that gates read/acquire behind auth, or lets the direct-source ref paths
rot untested, violates this spec.

## Scope

- In scope:
  - `packages/core/src/registry/**` (delete service + client; keep shared types).
  - `packages/core/src/marketplaces/**` (delete; discovery is hosted).
  - `packages/cli/src/skill-refs.ts` and any consumer of the TS registry: rewire
    onto the Rust native registry path.
  - Confirming/closing the Rust client gaps for `github:`/`file:`/url refs.
  - Cloud: confirm the `/v1/skills` surface is the client contract; add a
    `resolve` alias only if a gap is proven. No auth change to read/acquire.
- Out of scope (separate, undecided sunsets, each its own spec):
  - `packages/core/src/knowledge` (thread/handoff/feed projection). Depends on a
    nitrosend-dependency check first.
  - `packages/core/src/parser` SKILL.md frontmatter split (Rust already owns the
    YAML/manifest parsing).
  - Porting `doctor`/`list`/`init`/`new` CLI commands to Rust.
  - Self-serve publish / per-owner auth for the hosted index.

## Dependencies

- Supersedes the archived, failed `rust-ts-sunset-registry`; coordinate with its
  owner before deleting OSS TS.
- `rust-ts-sunset-marketplaces` (cancelled) folds into this; marketplaces was
  blocked behind registry.
- Runs in a checkout edited by concurrent agents; needs a clean window or an
  isolated worktree, since it deletes a module and rewires a CLI consumer.

## Assumptions

- The Rust registry client is feature-complete for resolve/fetch/verify/install
  against `/v1/skills` (verified: http.rs targets those routes).
- Cloud read/acquire are anonymous today (verified: public-api routes, cache
  `public`). Publish is admin-gated and stays that way.
- CONFIRMED (phase-1 audit): there is no client-side github fetch to port. The TS
  `url-add` command posts the repo URL to the cloud `/v1/index` endpoint and
  imports only `@runxhq/core/util` (not `core/registry`). GitHub fetch/index is
  already hosted; deleting `core/registry` does not affect `url-add`. The earlier
  "github-client gap" concern is void.
- CONFIRMED: the real consumer to rewire is `packages/cli/src/skill-refs.ts`. It
  uses `acquireRegistrySkill` (core/registry http-client) as the LIVE official-skill
  install path, wrapped in DIGEST-VERIFIED caching (`ensureOfficialSkillCached`,
  profile-state + X.yaml writing, sibling-ref rewriting), and it also consumes
  `@runxhq/core/marketplaces` (a dev-gated fixture adapter behind
  RUNX_ENABLE_FIXTURE_MARKETPLACE). Search already routes through the Rust CLI
  (`searchRegistryViaRustCli`).

## Risks

- Digest-verification regression (highest): `skill-refs.ts`'s official-skill cache
  verifies markdown/profile digests against `official-skills.lock.json` before
  writing. If the Rust delegation does not verify identically (or produces a
  different on-disk layout), the cutover silently weakens supply-chain integrity
  on the install path. Mitigation: confirm the Rust install verification + output
  contract first; keep `p1_ac2` (tampered input rejected) as a gate.
- Deleting a path nothing else covers: the github index path is hosted
  (`/v1/index`) and independent, so it is NOT at risk; the at-risk surface is the
  official-skill acquire path above. Mitigation: rewire and verify before delete.
- Auth-line regression: a later cloud change gates read/acquire. Mitigation: the
  invariant above plus an acceptance check that anonymous read/acquire works.
- Concurrency: deleting `core/registry` while other agents import it. Mitigation:
  rewire all consumers first (phase 1), delete last (phase 3), in a clean window.

## Acceptance

Profile: strict

Validation:
- `runx skill add <owner>/<skill>` resolves+installs from the hosted registry via Rust, digest-verified.
- `runx url-add github.com/<org>/<repo>` indexes via the cloud `/v1/index` (unchanged by this cutover).
- `runx skill add ./<path>` / local resolution still works.
- `rg -n "core/src/registry|@runxhq/core/registry|core/src/marketplaces|@runxhq/core/marketplaces|acquireRegistrySkill" packages` returns no live imports.
- `pnpm verify:fast` and `cargo nextest run --workspace --all-features` are green.

## Phase 1: Rewire skill-refs onto the Rust install path

Objective: `packages/cli/src/skill-refs.ts` no longer fetches/caches official
skills in TypeScript; it delegates official-skill acquire + on-disk install to the
Rust CLI, preserving digest verification. github (`/v1/index`) and search
(`searchRegistryViaRustCli`) already route correctly and are untouched.

This is security-sensitive: `ensureOfficialSkillCached` verifies the acquired
markdown/profile digests against `official-skills.lock.json` before writing. The
Rust install path must provide the same verification and produce the same on-disk
layout (SKILL.md, X.yaml, `.runx/profile.json`, sibling-ref rewriting) or the
delegation regresses security/behavior. Confirm the Rust install output contract
first; do this in a clean window or isolated worktree, not blind in a shared tree.

Changes:
- Replace `acquireRegistrySkill()` + the TS official-skill caching machinery in
  `skill-refs.ts` with a call to the Rust CLI install/acquire, returning the
  installed skill path. Preserve digest verification (delegate it to Rust).
- Remove the `@runxhq/core/marketplaces` consumer (the dev-gated fixture adapter)
  as part of the marketplaces deletion.
- Keep `searchRegistryViaRustCli` (already Rust-backed) and local/bundled
  resolution as-is.

Acceptance:
- [ ] `p1_ac1` command - official skill installs+runs via the Rust path
  - Command: `runx skill add runx/<skill> && runx skill <skill>`
  - Expected kind: `exit_code_zero`
- [ ] `p1_ac2` command - digest verification preserved (tampered lock/markdown rejected)
  - Expected kind: `manual`
- [ ] `p1_ac3` command - github index path still works via the cloud (unchanged)
  - Command: `runx url-add github.com/<org>/<repo>` (against a test api base)
  - Expected kind: `exit_code_zero`
- [ ] `p1_ac4` command - skill-refs no longer imports the TS registry/marketplaces
  - Command: `rg -n "@runxhq/core/registry|@runxhq/core/marketplaces|acquireRegistrySkill" packages/cli/src/skill-refs.ts`
  - Expected kind: `reviewed_output` (expect none)

## Phase 2: Confirm the cloud is the contract

Objective: the cloud `/v1/skills` surface is the documented client contract; no
new service is needed in OSS.

Changes:
- Document `/v1/skills` (search), `/v1/skills/{owner}/{name}` (read), and
  `/acquire` as the registry client API the Rust client targets.
- Only if a concrete gap is found (e.g., the client needs a single resolve call),
  add a thin `GET /v1/registry/{owner}/{name}/resolve` alias in the cloud API
  package. Do not change auth on read/acquire.

Acceptance:
- [ ] `p2_ac1` command - Rust client points at the hosted contract
  - Command: `rg -n "/v1/skills" oss/crates/runx-runtime/src/registry/http.rs`
  - Expected kind: `reviewed_output`
- [ ] `p2_ac2` command - read/acquire remain anonymous
  - Expected kind: `manual`

## Phase 3: Delete orphaned marketplaces; keep core/registry (DONE, revised)

REVISED after audit: `@runxhq/core/registry` is NOT redundant. The hosted cloud
imports it in ~24 source files (registry-publication, public-site-data,
skill-indexer, admin-persistence, public-api-service, mcp-hosted read model). It
is the cloud's registry-service library, so it is kept, not deleted. The
`SkillSearchResult` type stays there; the 3 OSS CLI importers use only that type.

`core/marketplaces` had no cloud importer and its only OSS consumer (the CLI
search) moved to Rust in phase 1, so it was orphaned and deleted.

Done:
- Deleted `packages/core/src/marketplaces` and its `./marketplaces` package export
  (merged 66755fb). Typecheck clean; fast tests green (only the pre-existing
  binary-env failures remain); no new failures.

Deferred (separate cloud-side effort, not this spec):
- Fully removing `core/registry` from OSS would require internalizing ~2.6k LOC
  into a cloud package across ~24 import sites. Track separately if desired.

Acceptance:
- [x] `p3_ac1` marketplaces module + export removed; no src importers remain.
- [x] `p3_ac2` typecheck clean; fast tests green apart from binary-env failures.

## Phase 4: Verify and close

Objective: prove the cutover end to end and reconcile the failed sunset spec.

Acceptance:
- [ ] `p4_ac1` command - Rust suite green
  - Command: `cargo nextest run --workspace --all-features`
  - Expected kind: `exit_code_zero`
- [ ] `p4_ac2` manual - the three install paths verified by a reviewer
  - Expected kind: `reviewed_output`
- [ ] `p4_ac3` manual - `rust-ts-sunset-registry` reconciled/closed with this as
  the superseding plan
  - Expected kind: `manual`

## Rollback

- Phases 1 and 2 are additive/rewiring and revert cleanly.
- Phase 3 is the only destructive step; keep it a single revertible commit. If an
  install path regresses, restore `packages/core/src/registry` from history and
  re-point `skill-refs.ts` at it while the Rust gap is closed.
- No cloud auth changes are made, so the adoption line cannot regress via this
  spec's own edits.

## Resulting TypeScript shape (after this spec)

- Removed: `core/registry`, `core/marketplaces`.
- Remains: generated `contracts`; thin `cli` shell plus the still-TS commands
  (`doctor`/`list`/`init`/`new`, pending a separate decision); `core` leftovers
  (`config`, `source`, `knowledge`, `parser`, `policy`, `util`); and the
  `authoring`/`create-skill`/`host-adapters`/`langchain`/`sdk-python` SDKs.
- Boundary after this spec: engine + install client = Rust; index/discovery/
  trust/publish = cloud; TypeScript = generated contracts + thin CLI + client-side
  config/source + authoring/integration SDKs.
