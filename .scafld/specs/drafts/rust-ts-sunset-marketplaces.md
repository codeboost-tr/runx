---
spec_version: '2.0'
task_id: rust-ts-sunset-marketplaces
created: '2026-05-18T00:00:00Z'
updated: '2026-05-22T12:00:00+10:00'
status: draft
harden_status: not_run
size: small
risk_level: medium
---

# TS sunset: marketplaces

## Current State

Status: draft, blocked
Current phase: discovery refresh after registry search-result migration
Next: reroute marketplace adapter/ref consumers before approval.
Reason: this draft describes a future deletion, not work that can be executed
against the current tree. A fresh 2026-05-22 source scan still finds 5 live
consumer/import files plus the marketplace source package marker and public
export with imports of `@runxhq/core/marketplaces` across CLI fixture
marketplace search, runtime-local SDK/install, and marketplace fixture tests.
The registry/CLI `SkillSearchResult` shape has moved to
`@runxhq/core/registry`, but marketplace adapter behavior remains live. The
prerequisite
`rust-ts-sunset-registry` is also archived as failed, not completed. Deleting
`packages/core/src/marketplaces/**` now would break current package exports and
live consumers.
Blockers:
- `rust-ts-sunset-registry` is archived with `status: failed`; the original
  dependency is not satisfied.
- CLI skill refs still import marketplace helpers and fixture adapters from
  `@runxhq/core/marketplaces`.
- Runtime-local skill install and SDK search/install surfaces still accept
  `MarketplaceAdapter` and call `resolveMarketplaceSkill` /
  `searchMarketplaceAdapters`.
- Focused tests still import marketplace fixtures/types directly.
- `packages/core/package.json` still exposes `./marketplaces`; remove it only
  after all importers are rerouted or retired by owning specs.
Allowed follow-up command: none while blocked; do not run `scafld harden rust-ts-sunset-marketplaces`.
Latest runner update: 2026-05-22T12:00:00+10:00 - child draft
`rust-ts-sunset-marketplaces-registry-search-result-ownership` moved
`SkillSearchResult` ownership to `@runxhq/core/registry`. Refreshed source scan
confirmed 5 live consumer/import files plus the marketplace source marker and
`./marketplaces` public export still reference marketplace adapter surfaces;
deletion remains blocked.
Review gate: not_started

## Summary

Future deletion target: remove `packages/core/src/marketplaces/` and the public
`@runxhq/core/marketplaces` export after all live consumers no longer depend on
it. The marketplaces domain is small, but it currently owns the shared
marketplace adapter contract, fixture marketplace adapter, and marketplace ref
classification. The `SkillSearchResult` model consumed by registry search
presentation is now registry-owned.

This spec must remain blocked until the shared search result model and
marketplace adapter contract have an explicit post-TypeScript owner. That owner
may be `runx-runtime::registry`, a contracts package surface, or a narrow
runtime/CLI boundary, but deletion must not add a compatibility shim or leave
the `@runxhq/core/marketplaces` subpath alive.

## Context

CWD: `.`

Packages:
- `@runxhq/core`
- `crates/runx-runtime` (or merged into registry)

Current TypeScript sources:
- `packages/core/src/marketplaces/**` (future deletion)

Files impacted:
- `packages/core/src/marketplaces/` (future deletion)
- `packages/core/package.json` (`"./marketplaces"` export removal)
- Any generated API-surface docs reflecting the removed export, if still
  present at execution time

Invariants:
- Marketplaces consumers (CLI surfaces, registry resolver, ai-search merge)
  have a Rust path.
- No compatibility shim, re-export, fallback adapter, or legacy TypeScript
  package surface remains after deletion.
- `SkillSearchResult` ownership is explicit before deletion; registry search
  now imports it from the registry package.

Current live importers found in the 2026-05-22 post-migration source scan:
- `packages/cli/src/skill-refs.ts`
- `packages/runtime-local/src/runner-local/skill-install.ts`
- `packages/runtime-local/src/sdk/index.ts`
- `tests/skill-add.test.ts`
- `tests/skill-add-profile-metadata.test.ts`

Still-public export:
- `packages/core/package.json` exports `./marketplaces` to
  `./dist/src/marketplaces/index.{d.ts,js}`.

## Objectives

- Keep this draft honest about current blockers.
- Track the exact public export removal target:
  `packages/core/package.json` `exports["./marketplaces"]`.
- Require a fresh source scan before any deletion attempt.
- Delete TS marketplaces only after the marketplace adapter/search-result
  contracts and all consumers have moved to their post-TypeScript owner.

## Scope

In scope:
- Future TS marketplaces deletion plan.
- Future public export removal for `@runxhq/core/marketplaces`.
- Importer verification and deletion gating.

Out of scope:
- Marketplace functionality changes.
- Rerouting remaining CLI/runtime-local/SDK marketplace adapter consumers.
- Moving marketplace adapter contracts to a new owner.
- Legacy import compatibility, package shims, or fallback adapters.

## Dependencies

- A completed registry ownership/cutover path; the current
  `rust-ts-sunset-registry` archive entry is failed and cannot satisfy this
  dependency.
- A `rust-marketplaces-port` spec, a merger into `runx-runtime::registry`, or a
  contracts/runtime boundary spec that owns `SkillSearchResult`,
  `MarketplaceAdapter`, marketplace ref parsing, and fixture marketplace test
  setup.
- Remaining marketplace adapter/ref consumers must move away from
  `@runxhq/core/marketplaces` before this deletion can be approved.
- A fresh importer scan immediately before approval.

## Importer Census

Checked on 2026-05-22:

```bash
rg -l "@runxhq/core/marketplaces|\\.\\./marketplaces/index\\.js|packages/core/src/marketplaces|\\\"\\./marketplaces\\\"" packages tests --glob '!**/dist/**' --glob '!node_modules' | sort
rg -n "@runxhq/core/marketplaces|\\.\\./marketplaces/index\\.js|packages/core/src/marketplaces|\\\"\\./marketplaces\\\"" packages tests --glob '!**/dist/**' --glob '!node_modules'
```

Observed results:
- 5 live consumer/import files remain after excluding generated `dist` output.
- `packages/core/package.json` still exposes `./marketplaces`.
- `packages/core/src/marketplaces/index.ts` remains the source implementation
  and declares the public `@runxhq/core/marketplaces` package marker.

Live consumer/import files:
- `packages/cli/src/skill-refs.ts`
  - Imports `createFixtureMarketplaceAdapter` and `searchMarketplaceAdapters`
    for CLI fixture marketplace lookup.
- `packages/runtime-local/src/runner-local/skill-install.ts`
  - Imports `isMarketplaceRef`, `resolveMarketplaceSkill`, and
    `MarketplaceAdapter` for install resolution.
- `packages/runtime-local/src/sdk/index.ts`
  - Imports fixture/search marketplace adapters and marketplace/search-result
    types for SDK search/install surfaces.
- `tests/skill-add.test.ts`
  - Imports marketplace adapter/result types for invalid marketplace fixture
    coverage.
- `tests/skill-add-profile-metadata.test.ts`
  - Imports fixture marketplace adapter and marketplace adapter/result types for
    profile metadata install coverage.

Deletion gate:
- Blocked. Do not delete `packages/core/src/marketplaces/**`, remove
  `packages/core/package.json` `exports["./marketplaces"]`, or run
  `scafld harden rust-ts-sunset-marketplaces` while any consumer/import file
  above remains.

## Acceptance

Profile: standard

Definition of done:
- [x] `dod1` Marketplace/core importer census is refreshed against the current
  tree.
- [x] `dod2` Parent deletion remains explicitly blocked while consumers remain.
- [x] `dod3` Registry/search-result ownership migration is split into a child
  draft.
- [x] `dod4` Marketplace implementation and public export remain present while
  deletion stays blocked.

Validation:
- [x] `v1` Scafld validates this spec.
  - Command: `scafld validate rust-ts-sunset-marketplaces --json`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: returned `{"ok":true,...,"valid":true}`.
- [x] `v2` Marketplace importer census remains non-empty and blocks deletion.
  - Command: `rg -l "@runxhq/core/marketplaces|\\.\\./marketplaces/index\\.js|packages/core/src/marketplaces|\\\"\\./marketplaces\\\"" packages tests --glob '!**/dist/**' --glob '!node_modules' | sort`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: listed 5 live consumer/import files, the marketplace source
    implementation, and `packages/core/package.json`.
- [x] `v3` Public marketplace export remains present.
  - Command: `rg -n "\"\\./marketplaces\"" packages/core/package.json`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: `packages/core/package.json` still exports `./marketplaces`.

## Open Questions

- Whether marketplaces ships as its own Rust module, folds into
  `runx-runtime::registry`, or is split between contracts-owned types and
  runtime/CLI-owned adapters.
- Which spec owns fixture marketplace test setup after the TS package is
  deleted?
