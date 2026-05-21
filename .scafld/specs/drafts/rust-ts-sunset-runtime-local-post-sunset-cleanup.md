---
spec_version: '2.0'
task_id: rust-ts-sunset-runtime-local-post-sunset-cleanup
created: '2026-05-21T13:42:18Z'
updated: '2026-05-21T13:42:18Z'
status: draft
harden_status: not_run
size: large
risk_level: high
---

# Runtime-local sunset: post-sunset cleanup

## Current State

Status: draft
Current phase: blocked
Next: wait for `rust-ts-sunset-runtime-local` completion
Reason: this is a follow-up cleanup ledger for residue after the runtime-local
and adapters sunset has landed. It must not become a second place to perform
the cutover itself.
Blockers: `rust-ts-sunset-runtime-local` must be completed and archived with
review pass evidence. External adapter/plugin authoring and embedded/cloud
binding disposition must already be settled by the prerequisite sunset work.
Allowed follow-up command: `none until rust-ts-sunset-runtime-local is complete`
Latest runner update: 2026-05-21T13:42:18Z
Review gate: not_started

Guardrail: this spec must not resurrect runtime-local/adapters through tests,
scripts, docs, fixtures, API-surface references, package metadata, path aliases,
shim packages, v2 package names, or compatibility adapters. It only removes
stale post-sunset references and tightens negative checks after the parent
deletion has landed.

## Summary

Clean up the workspace after `@runxhq/runtime-local` and `@runxhq/adapters`
have been deleted or fully sunset. This spec owns the post-cutover residue:
stale API-surface docs, orphaned fixture/oracle scripts, release/build checks,
legacy package references in generated config or lockfiles, archived test
metadata, and active docs that still describe the deleted packages as live
surfaces.

This spec is intentionally downstream of `rust-ts-sunset-runtime-local`. The
parent sunset owns deletion of `packages/runtime-local/`, deletion of
`packages/adapters/`, importer routing, package manifest/path alias removal,
and the proof that no surviving local caller imports the deleted packages.

## Context

The 2026-05-21 cleanup census before sunset still found broad active residue:
- root tests importing runtime-local/adapters;
- oracle fixture generators encoding TypeScript adapter paths;
- `tsconfig.base.json`, `vitest.workspace-aliases.ts`, root `package.json`,
  and `pnpm-lock.yaml` references;
- `docs/api-surface.md` tables exposing deleted package exports;
- fixtures and skill docs naming legacy package paths;
- active sunset/spec docs that must either archive with their parent work or be
  rewritten to describe the completed state.

Those are not all post-sunset tasks. Many are prerequisites owned by
`rust-ts-sunset-runtime-local`. This spec starts only after that prerequisite
is complete, then removes the remaining documentation, generator, and metadata
debris so the repository cannot drift back into a dual-runtime shape.

## Objectives

- Confirm the runtime-local/adapters sunset completed and no package directory,
  workspace dependency, path alias, or surviving-package import remains.
- Remove or archive stale API-surface and generated documentation for deleted
  package exports.
- Delete or retire obsolete TypeScript oracle generators whose outputs are now
  durable Rust/contract fixtures.
- Remove release/build/check logic that special-cases deleted packages.
- Refresh fixture inventories so active fixtures do not refer to deleted package
  paths except as explicit negative-history cases.
- Keep the TypeScript survivorship rule intact: TypeScript may remain for
  contracts, clients, cloud/product integration code, authoring, host adapters,
  and protocol helpers, but not for trusted local runtime fallback.

## Scope

In scope:
- Post-sunset documentation cleanup.
- Post-sunset fixture and oracle-generator cleanup.
- Build, release, check, and API-surface metadata cleanup.
- Negative scans proving deleted package names do not remain in active
  implementation surfaces.
- Archiving or rewriting active specs whose only remaining purpose was to track
  the sunset.

Out of scope:
- Deleting `packages/runtime-local/` or `packages/adapters/`; owned by
  `rust-ts-sunset-runtime-local`.
- Migrating tests or live importers from runtime-local/adapters; owned by
  `rust-ts-sunset-runtime-local` and its smaller importer specs.
- Defining or implementing the external adapter/plugin protocol; owned by
  `external-adapter-plugin-protocol-v1`.
- Classifying embedded/cloud runtime-local SDK consumers; owned by
  `embedded-sdk-migration-story`.
- Reintroducing a compatibility package, v2 shim, alias, or TypeScript local
  runtime fallback.

## Dependencies

- `rust-ts-sunset-runtime-local` must be completed with review gate pass before
  this spec can be approved or executed. Draft, active, blocked, or partially
  completed parent state is not sufficient.
- This is a post-sunset hygiene pass only. If any active caller still requires
  `@runxhq/runtime-local`, `@runxhq/adapters`, `packages/runtime-local`, or
  `packages/adapters`, repair or reopen the sunset path instead of running this
  cleanup.
- `ts-extension-survivorship-boundary` must be completed or superseded by a
  stricter boundary doc.
- `external-adapter-plugin-protocol-v1` and `embedded-sdk-migration-story` must
  already be settled by the parent sunset path: custom adapter authoring and
  cloud/embedded binding disposition are preconditions, not work introduced
  here.

## Touchpoints

- `oss/docs/api-surface.md`
- `oss/docs/ts-interop-boundary.md`
- `oss/docs/rust-kernel-architecture.md`
- `oss/README.md`
- `oss/scripts/generate-a2a-adapter-fixtures.ts`
- `oss/scripts/generate-agent-adapter-fixtures.ts`
- `oss/scripts/generate-runtime-catalog-adapter-oracles.ts`
- `oss/scripts/generate-runtime-mcp-oracles.ts`
- `oss/scripts/generate-cli-feature-parity.ts`
- `oss/scripts/check-rust-cli-cutover-negative.mjs`
- `oss/scripts/check-rust-cli-release-artifacts.ts`
- `oss/fixtures/**`
- `oss/skills/**`
- `oss/package.json`
- `oss/pnpm-lock.yaml`
- `oss/tsconfig.base.json`
- `oss/vitest.workspace-aliases.ts`

## Acceptance

Profile: strict

Definition of done:
- [ ] `dod1` `rust-ts-sunset-runtime-local` is completed/archived and the
  package directories are gone.
- [ ] `dod2` Active non-spec sources have zero references to
  `@runxhq/runtime-local`, `@runxhq/adapters`, `packages/runtime-local`, or
  `packages/adapters`.
- [ ] `dod3` API-surface docs no longer expose deleted runtime-local/adapters
  package exports as live package surfaces.
- [ ] `dod4` Obsolete TypeScript oracle generators are deleted, archived, or
  rewritten to operate only on durable Rust/contract fixtures.
- [ ] `dod5` Release/build/check scripts no longer special-case deleted
  packages except in explicitly named negative-history fixtures.
- [ ] `dod6` Active docs describe the final boundary: Rust trusted local
  runtime, TypeScript survivorship through stable contracts/protocols, and no
  local runtime fallback.
- [ ] `dod7` No compatibility package, v2 alias, path alias, or shim is present
  for runtime-local/adapters.

Validation:
- [ ] `v1` Scafld validates this spec.
  - Command: `scafld validate rust-ts-sunset-runtime-local-post-sunset-cleanup --json`
  - Expected kind: `exit_code_zero`
  - Status: pending
  - Evidence: none
- [ ] `v2` Deleted package directories stay deleted.
  - Command: `test ! -d packages/runtime-local && test ! -d packages/adapters`
  - Expected kind: `exit_code_zero`
  - Status: pending
  - Evidence: none
- [ ] `v3` Active non-spec surfaces have no deleted package references.
  - Command: `! rg -n "@runxhq/(runtime-local|adapters)|packages/(runtime-local|adapters)" . --glob '!.scafld/specs/**' --glob '!**/dist/**' --glob '!node_modules/**'`
  - Expected kind: `exit_code_zero`
  - Status: pending
  - Evidence: none
- [ ] `v4` Boundary guardrail passes.
  - Command: `node scripts/check-boundaries.mjs`
  - Expected kind: `exit_code_zero`
  - Status: pending
  - Evidence: none
- [ ] `v5` Workspace type/build metadata no longer resolves deleted packages.
  - Command: `! rg -n "@runxhq/(runtime-local|adapters)|packages/(runtime-local|adapters)" package.json pnpm-lock.yaml tsconfig.base.json vitest.workspace-aliases.ts`
  - Expected kind: `exit_code_zero`
  - Status: pending
  - Evidence: none
- [ ] `v6` Docs expose no live deleted package API surface.
  - Command: `! rg -n "## @runxhq/(runtime-local|adapters)|@runxhq/(runtime-local|adapters).+dist/" docs README.md`
  - Expected kind: `exit_code_zero`
  - Status: pending
  - Evidence: none

## Phase 1: Completion Gate

Goal: prove this is post-sunset work, not a hidden cutover.

Status: pending
Dependencies: `rust-ts-sunset-runtime-local`

Changes:
- Verify `rust-ts-sunset-runtime-local` is completed or archived with review
  pass evidence.
- Verify `packages/runtime-local/` and `packages/adapters/` are absent.
- Verify `ts-extension-survivorship-boundary`,
  `external-adapter-plugin-protocol-v1`, and `embedded-sdk-migration-story`
  have either completed, archived, or explicitly recorded that no post-sunset
  cleanup depends on their unfinished work.

Acceptance:
- If any prerequisite is incomplete, stop. Do not use this spec to perform
  importer migration or package deletion.

## Phase 2: Documentation And API Surface

Goal: remove live documentation for deleted package exports.

Status: pending
Dependencies: Phase 1

Changes:
- Remove `@runxhq/runtime-local` and `@runxhq/adapters` live export sections
  from `docs/api-surface.md`.
- Update README and boundary docs from "sunset pending" language to completed
  final-state language.
- Archive or rewrite active specs that only existed to track the runtime-local
  sunset.

Acceptance:
- Active docs describe only surviving boundaries and historical archive links.

## Phase 3: Fixture And Generator Retirement

Goal: remove TypeScript oracle residue after durable Rust/contract fixtures own
behavior.

Status: pending
Dependencies: Phase 1

Changes:
- Delete, archive, or rewrite adapter oracle generators that reference deleted
  package paths.
- Remove stale fixture metadata that points at runtime-local/adapters source
  paths.
- Keep explicit negative-history fixtures only when their names and assertions
  make the legacy package reference intentional.

Acceptance:
- No active fixture or generator depends on a deleted TypeScript package path.

## Phase 4: Build And Guardrail Cleanup

Goal: remove deleted packages from workspace mechanics.

Status: pending
Dependencies: Phase 1

Changes:
- Remove stale package references from root manifest, lockfile, TypeScript path
  aliases, Vitest aliases, release artifact checks, cutover-negative checks,
  and CLI feature parity generation.
- Keep `scripts/check-boundaries.mjs` strict about forbidden compatibility
  packages and surviving-package imports.

Acceptance:
- Boundary checks pass and there is no package manager or compiler resolution
  path back to deleted packages.

## Phase 5: Final Census

Goal: prove the cleanup stayed complete.

Status: pending
Dependencies: Phases 2, 3, 4

Changes:
- Run the active-surface negative scans.
- Run the boundary guardrail.
- Run focused docs/metadata checks.

Acceptance:
- Deleted package names appear only in archived specs, external release notes
  intentionally documenting history, or this spec's own review history.

## Rollback

If cleanup removes a fixture or script that still owns unique behavior, restore
it only as a neutral Rust/contract fixture generator. Do not restore imports
from `@runxhq/runtime-local`, `@runxhq/adapters`, or deleted package source
paths.

## Review

Review must reject any cleanup that silently performs the runtime-local sunset
itself, any cleanup that hides unfinished embedded/cloud or adapter-protocol
work, and any cleanup that reintroduces a compatibility shim for deleted
packages.

## Origin

User review on 2026-05-21 identified that the explicit TypeScript survivorship
boundary exposes a large post-cutover residue cleanup wave. This spec records
that cleanup as work to execute only after `rust-ts-sunset-runtime-local` has
completed.
