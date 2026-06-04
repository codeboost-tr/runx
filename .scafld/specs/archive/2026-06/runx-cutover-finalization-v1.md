---
spec_version: '2.0'
task_id: runx-cutover-finalization-v1
created: '2026-06-03T00:00:00Z'
updated: '2026-06-03T16:21:16Z'
status: completed
harden_status: not_run
size: large
risk_level: high
---

# runx cutover finalization: final shape + S-tier

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-06-03T16:21:16Z
Review gate: pass

## Summary

The runx OSS cutover to "one governed core, many fronts" is substantially done and
high quality. This spec closes the residual gap between the current tree and the
final ultimate shape + S-tier code, in four phases ordered low-risk-first:

1. S-tier de-duplication + correctness (two MAJOR DRY/parity issues + small nits).
2. Doc reconciliation (one MAJOR stale feature list + minor citation drift).
3. Generic-contracts finalization (remove payment-specific shapes from the
   generic `runx-contracts` crate behind the generic effect seam).
4. Green every CI gate and close.

It is deliberately scoped to final-shape cutover work. The real-rail build, the
deferred finality-receipt wiring, the OpenAPI front, and the cloud-to-kernel
bridge stay out of scope (future / separate specs).

## Objectives

- The generic crates (`runx-contracts`, `runx-runtime`, `runx-core`) carry no
  payment-specific shapes; payment is one opaque effect family. The domain-free
  gate covers `runx-contracts/src` too.
- Every cross-language invariant is single-source or pinned by a test: the tool
  `source_hash` algorithm agrees byte-for-byte across Rust and TS; the
  story-milestone vocabulary has one canonical definition.
- The docs match the code: feature lists, citations, and architecture notes are
  accurate at HEAD.
- All nine CI gates are green, with the payment/source fixtures and
  `@runxhq/contracts` mirror migrated to the clean final shape.

## Invariant (must-not-regress)

- **Skill/source surface is frozen except the directed contract hard cutover.**
  No `SourceKind` variant, runner-manifest ABI,
  `external-adapter`/`thread-outbox-provider` protocol, or CLI JSON shape is
  removed or renamed. Phase 3 is a hard greenfield contract cutover: old
  payment-specific authority/proof wire names are removed from generic contracts,
  schemas, fixtures, and the `@runxhq/contracts` mirror with no legacy shim.
- **Kernel stays payment-free.** `runx-runtime/src` + `runx-core/src` keep zero
  payment/spend/settlement/x402/rail identifiers (the `--final` gate in
  `scripts/check-runtime-cutover-legacy.mjs`); after phase 3 the same gate covers
  `runx-contracts/src`.
- **No in-kernel provider HTTP clients return.** Only the sanctioned network
  surfaces stay (governed HTTP front `runtime_http`, registry client, inbound MCP
  HTTP server, the in-binary agent resolver).
- **`@runxhq/core` stays `private: true` and has no active importer.** OSS and
  cloud use the Rust/native contracts or the cloud-owned `@runx/protocol`
  package; no workspace alias or dependency points cloud back at `@runxhq/core`.

## Scope

- In scope:
  - De-dup the story-milestone vocabulary (`tools/thread/story.ts` vs
    `tools/outbox/story.ts`).
  - Unify or pin the tool `source_hash` algorithm across `build.rs` and
    `authoring-utils.ts` (add a cross-language agreement test).
  - Small Rust nits: drop the now-dead `provider.rs` re-export, extract the
    duplicated `EffectReceiptRequest` builder in `authority.rs`, fix the
    write-only `_message` test field, remove two empty leftover dirs.
- Doc fixes: `rust-kernel-architecture.md` feature lists; two
  `governed-execution-layer.md` citations; crate README runtime feature lists;
  refresh the effect-kernel spec's stale "Current State" note.
  - Phase 3: migrate payment-specific shapes out of `runx-contracts` behind the
    generic `AuthorityEffectGuard` / `ProofKind::EffectFinality` seam; extend
    the domain-free gate to `runx-contracts/src`; finish the
    `EffectSupervisor` trait generalization (`naming-1`).
  - Hard-cut cloud `@runxhq/core` importers to the first-class
    `@runx/protocol` package and add a cloud boundary guard forbidding the
    retired package.
  - Phase 4: run and green all CI gates.
- Out of scope (future / separate specs):
  - Wiring `EffectFinalityReceipt` runtime emission (needs a real
    async/provisional rail — `payment-rails.md` Phase 1).
  - Real rails (x402/Stripe/MPP), the cloud-to-kernel bridge, the OpenAPI front
    (`governed-execution-layer.md` items 11-15, `payment-rails.md`).

## Dependencies

- The effect-kernel spec `oss/.scafld/specs/archive/2026-05/runx-effect-kernel-v1.md`
  (Phase 4 is explicitly deferred; phase 3 here is that deferred cleanup).
- The boundary source of truth `oss/docs/ts-interop-boundary.md` and
  `../../plans/governed-execution-layer.md`.
- Phase 3 touches public contract schemas; coordinate the `@runxhq/contracts` TS
  mirror + fixture cross-validation, and obtain approval (`public_api_changes`).

## Assumptions

- Verified at HEAD f205f676 (deep review, 2026-06-03): boundary clean, kernel
  payment-free, `@runxhq/core` `private: true` with zero OSS importers, all 10
  `SourceKind` variants intact, no coverage regression, inlining
  behavior-preserving, cloud duplications already collapsed (313bb5d).
- The two MAJOR findings (`dry-1` story duplication, `dry-2` hash parity) and the
  MAJOR `residue-1` (contracts payment shapes) survived adversarial verification.
- The app is greenfield, so `runx-contracts` can hard-cut to generic
  `AuthorityEffectLimit` / `ProofKind::{EffectEvidence, EffectFinality}` shapes
  without preserving deprecated payment-specific wire names.

## Risks

- **Fixture/schema churn (highest, phase 3).** `AuthorityBounds`, `ProofKind`, and
  effect limits are serialized into authorities and sealed receipts. The hard
  cutover intentionally refreshes greenfield fixtures and schemas. Mitigation:
  migrate source fixtures and the TS mirror in the same change; cross-validate
  fixtures (`pnpm fixtures:*:check`) and dogfood payment skills after migration.
- **Hash-parity fix masking a real divergence (phase 1, `dry-2`).** If the TS and
  Rust scanners already disagree on some tool, unifying them changes a manifest
  `source_hash`. Mitigation: add the agreement test FIRST against current tools;
  if it fails, that is a pre-existing bug to surface, not silently absorb.
- **Over-eager dedup breaking the invariant map (`dry-1`).** The
  `LEGACY_STORY_MILESTONE_ID_MAP` is invariant-critical. Mitigation: single-source
  it by re-export, not by hand-editing two copies; keep behavior byte-identical.
- **Cloud protocol fork drift.** The hard cutover promotes the surviving pure
  helper domains to `cloud/packages/protocol`. Mitigation: cloud boundary checks
  forbid `@runxhq/core` imports, and server typecheck exercises the promoted
  package exports.

## Acceptance

Profile: strict

Validation:
- No retired rail-supervisor vocabulary remains in the crates.
- The generic crates carry no payment shapes: the `--final` domain-free scan, once
  extended, passes over `runx-contracts/src` as well as `runx-runtime/src` +
  `runx-core/src`.
- `tools/thread/story.ts` has no hand-maintained duplicate of
  `tools/outbox/story.ts`; one canonical milestone vocabulary.
- A cross-language test pins the tool `source_hash` (Rust `build.rs` ==
  TS `authoring-utils.ts`) on a fixture covering escaped/mixed quotes.
- Rust and TS/fixture gates are run in arm64 Linux Docker toolchains because this
  local macOS session rejects newly built Mach-O test binaries at the system
  policy layer:
  `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features
  -- -D warnings`, `cargo nextest run --workspace --all-features`, and
  `cargo test --workspace --all-features --doc`; plus Node 20 `pnpm
  verify:fast` and `pnpm fixtures:harness:check`. The license-boundary checks
  are green.
- The `issue-to-pr` graph still runs end-to-end; no `SourceKind`/protocol changed.
- The cloud protocol helper cutover is verified from the OSS spec root by
  checking `../cloud/packages/protocol/package.json` declares `@runx/protocol`,
  cloud package manifests import that package instead of `@runxhq/core`, and
  `../cloud/scripts/check-workspace-boundary.mjs` uses
  `FORBIDDEN_LEGACY_CORE_PACKAGE_PATTERN` to forbid retired `@runxhq/core`
  imports with `@runx/protocol` replacement guidance.

## Phase 1: S-tier de-duplication and correctness

Status: completed
Dependencies: none

Objective: remove the two MAJOR duplications and the small Rust nits the review

Changes:
- `dry-1` (MAJOR): make `oss/tools/thread/story.ts` re-export the milestone vocabulary from `oss/tools/outbox/story.ts` (or hoist both to one shared tool-local module). Delete the verbatim 51-line copy. Keep `LEGACY_STORY_MILESTONE_ID_MAP` single-source.
- `dry-2` (MAJOR): pick ONE canonical tool-source hasher. Preferred: Rust (`crates/runx-runtime/src/tool_catalogs/build.rs:253-419`) is canonical; the TS doctor (`packages/cli/src/authoring-utils.ts`) shells out to `runx` for the hash instead of re-scanning. If two implementations must remain, align the TS import-specifier scanner to the Rust char-scanner's escape/quote semantics and add a cross-language fixture test that pins them byte-for-byte.
- `indirection-1` (nit): delete `provider.rs:3` (`pub(super) use super::{github_pull_request_number, github_repository};`) and import directly from `super::` at the consumer (`target_runner/pull_request.rs:17`); consider folding the 25-line `provider.rs` into `target_runner.rs`.
- `dry-3` (nit): in `crates/runx-runtime/src/execution/runner/authority.rs:87-124` extract the identical 8-field `EffectReceiptRequest` builder shared by `finalize_effect_output_before_success` and `persist_effect_state_for_step`.
- `test-1` (nit): in `crates/runx-runtime/src/receipts/seal.rs:967-991` drop the write-only `_message` field (or assert on it).
- `kernel-2` cleanup (nit): remove the empty leftover dirs `crates/runx-runtime/src/payment` and `crates/runx-core/src/policy/payment_authority`.

Acceptance:
- [x] `p1_ac1` command - story vocabulary is single-source
  - Command: `node -e "const a=require('fs').readFileSync('tools/outbox/story.ts','utf8');const b=require('fs').readFileSync('tools/thread/story.ts','utf8');process.exit(b.includes('LEGACY_STORY_MILESTONE_ID_MAP')&&!b.includes('export { ')&&!b.includes('from \"../outbox/story')?1:0)"`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-36
- [x] `p1_ac2` command - tool source-hash parity is pinned
  - Command: `pnpm vitest run packages/cli/src/authoring-utils`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-37
- [x] `p1_ac3` command - clippy clean after the Rust nits
  - Command: `docker volume create runx-cargo-cache >/dev/null && docker volume create runx-rust-target >/dev/null && docker run --rm -v "$PWD":/work -v runx-cargo-cache:/usr/local/cargo -v runx-rust-target:/cargo-target -w /work rust:1.95-bookworm bash -c 'set -euo pipefail; export PATH=/usr/local/cargo/bin:/usr/local/rustup/toolchains/1.95.0-aarch64-unknown-linux-gnu/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin; apt-get update >/dev/null && apt-get install -y pkg-config libssl-dev ca-certificates >/dev/null; rustup component add clippy >/dev/null; export CARGO_TARGET_DIR=/cargo-target; cargo clippy --manifest-path crates/Cargo.toml -p runx-runtime --lib -- -D warnings'`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-38
- [x] `p1_ac4` command - issue-to-pr graph still seals (story tools unaffected)
  - Command: `docker volume create runx-cargo-cache >/dev/null && docker volume create runx-rust-target >/dev/null && docker run --rm -v "$PWD":/work -v runx-cargo-cache:/usr/local/cargo -v runx-rust-target:/cargo-target -w /work rust:1.95-bookworm bash -c 'set -euo pipefail; export PATH=/usr/local/cargo/bin:/usr/local/rustup/toolchains/1.95.0-aarch64-unknown-linux-gnu/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin; apt-get update >/dev/null && apt-get install -y pkg-config libssl-dev ca-certificates >/dev/null; export CARGO_TARGET_DIR=/cargo-target; cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features cli-tool --test integration issue_to_pr'`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-39

## Phase 2: Doc reconciliation

Status: completed
Dependencies: none

Objective: docs match code at HEAD.

Changes:
- `drift-1` (MAJOR): `oss/docs/rust-kernel-architecture.md:124,:204,:690`, `oss/crates/README.md`, and `oss/crates/runx-runtime/README.md` — replace the stale `cli-tool, mcp, a2a, agent, catalog` lists with the real set (`cli-tool, mcp, mcp-http-server, a2a, agent, catalog, external-adapter, http`), flagging `a2a` as contract-defined but not enabled in `runx-cli`.
- `drift-2` (minor): `../../plans/governed-execution-layer.md:32` — repoint the `finalize_output` citation to `runx-pay/src/runtime.rs:332-362`.
- `drift-3` (nit): `../../plans/governed-execution-layer.md:131` — fix the Cargo anchor to `runx-cli/Cargo.toml:29`.
- Refresh the effect-kernel spec's "Current State" note where it is stale on the completed rail-supervisor removal (`kernel-1`).
- Do NOT "fix" the cloud agent-runner "single-shot" wording: `tool-loop.ts` was deleted (313bb5d), so single-shot is now correct (`drift-ok-1`).

Acceptance:
- [x] `p2_ac1` command - no stale feature list remains
  - Command: `rg -n "cli-tool, mcp, a2a, agent, catalog" docs/ crates/README.md crates/runx-runtime/README.md`
  - Expected kind: `no_matches`
  - Status: pass
  - Evidence: output was empty
  - Source event: entry-44
- [x] `p2_ac2` command - citations in governed-execution-layer.md resolve to the cited symbols
  - Command: `node -e "const fs=require('fs');const plan=fs.readFileSync('../plans/governed-execution-layer.md','utf8');const runtime=fs.readFileSync('crates/runx-pay/src/runtime.rs','utf8');const cargo=fs.readFileSync('crates/runx-cli/Cargo.toml','utf8');if(!plan.includes('oss/crates/runx-pay/src/runtime.rs:332-362')||!runtime.includes('fn finalize_output')||!plan.includes('oss/crates/runx-cli/Cargo.toml:29')||!cargo.includes('features = [\"cli-tool\", \"catalog\", \"mcp\", \"mcp-http-server\", \"external-adapter\", \"agent\", \"http\"]'))process.exit(1)"`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-45

## Phase 3: Generic-contracts finalization (the one real final-shape gap)

Status: completed
Dependencies: none

Objective: the generic `runx-contracts` crate carries no payment-specific shapes;

Changes:
- `residue-1` (MAJOR): remove the payment-specific shapes from `crates/runx-contracts/src/authority.rs` and `crates/runx-contracts/src/reference.rs`. Payment expresses its bounds through `AuthorityBounds.effect_limits[]` / `AuthorityEffectLimit` and proof through `ProofKind::EffectEvidence` / `ProofKind::EffectFinality`; `runx-pay` owns the payment-specific interpretation.
- `naming-1` (minor): finish the `EffectSupervisor` generalization (`crates/runx-pay/src/runtime.rs:42`) so the method/types are family-agnostic (`supervise(request) -> evidence` over an opaque payload), not family-finality-only. (Resolves the "generic name over domain-only types" half-abstraction.)
- Extend `scripts/check-runtime-cutover-legacy.mjs` `--final` domain-free scan to include `crates/runx-contracts/src`, so the contracts crate cannot reaccumulate payment identifiers.

Acceptance:
- [x] `p3_ac1` command - generic crates are payment-free under the widened gate
  - Command: `node scripts/check-runtime-cutover-legacy.mjs --final`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-57
- [x] `p3_ac2` command - fixtures and Rust tests validate the hard migration
  - Command: `docker volume create runx-cargo-cache >/dev/null && docker volume create runx-rust-target >/dev/null && docker volume create runx-pnpm-store >/dev/null && docker run --rm -v "$PWD":/src:ro -v runx-cargo-cache:/usr/local/cargo -v runx-rust-target:/cargo-target -v runx-pnpm-store:/pnpm-store -w /tmp rust:1.95-bookworm bash -c 'set -euo pipefail; export PATH=/usr/local/cargo/bin:/usr/local/rustup/toolchains/1.95.0-aarch64-unknown-linux-gnu/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin; apt-get update >/dev/null && apt-get install -y pkg-config libssl-dev ca-certificates curl tar gnupg >/dev/null; curl -fsSL https://deb.nodesource.com/setup_20.x | bash - >/dev/null; apt-get install -y nodejs >/dev/null; npm install -g pnpm@10.18.2 >/dev/null; mkdir -p /work; tar -C /src --exclude=./node_modules --exclude=./crates/target --exclude=./.git -cf - . | tar -C /work -xf -; cd /work; rm -rf crates/target && ln -s /cargo-target crates/target; export CARGO_TARGET_DIR=/cargo-target; pnpm config set store-dir /pnpm-store; pnpm install --frozen-lockfile --prefer-offline; pnpm fixtures:harness:check' && docker run --rm -v "$PWD":/work -v runx-cargo-cache:/usr/local/cargo -v runx-rust-target:/cargo-target -w /work rust:1.95-bookworm bash -c 'set -euo pipefail; export PATH=/usr/local/cargo/bin:/usr/local/rustup/toolchains/1.95.0-aarch64-unknown-linux-gnu/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin; apt-get update >/dev/null && apt-get install -y pkg-config libssl-dev ca-certificates curl tar nodejs >/dev/null; if ! command -v cargo-nextest >/dev/null 2>&1 || ! cargo nextest --version | grep -q "0.9.137"; then rm -f /usr/local/cargo/bin/cargo-nextest; curl -LsSf https://get.nexte.st/0.9.137/linux-arm | tar zxf - -C /usr/local/cargo/bin; fi; export CARGO_TARGET_DIR=/cargo-target; cargo nextest run --manifest-path crates/Cargo.toml --workspace --all-features'`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-58
- [x] `p3_ac3` command - the @runxhq/contracts TS mirror + fixtures still cross-validate
  - Command: `docker volume create runx-cargo-cache >/dev/null && docker volume create runx-rust-target >/dev/null && docker volume create runx-pnpm-store >/dev/null && docker run --rm -v "$PWD":/src:ro -v runx-cargo-cache:/usr/local/cargo -v runx-rust-target:/cargo-target -v runx-pnpm-store:/pnpm-store -w /tmp rust:1.95-bookworm bash -c 'set -euo pipefail; export PATH=/usr/local/cargo/bin:/usr/local/rustup/toolchains/1.95.0-aarch64-unknown-linux-gnu/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin; apt-get update >/dev/null && apt-get install -y pkg-config libssl-dev ca-certificates curl tar gnupg >/dev/null; curl -fsSL https://deb.nodesource.com/setup_20.x | bash - >/dev/null; apt-get install -y nodejs >/dev/null; npm install -g pnpm@10.18.2 >/dev/null; mkdir -p /work; tar -C /src --exclude=./node_modules --exclude=./crates/target --exclude=./.git -cf - . | tar -C /work -xf -; cd /work; rm -rf crates/target && ln -s /cargo-target crates/target; export CARGO_TARGET_DIR=/cargo-target; pnpm config set store-dir /pnpm-store; pnpm install --frozen-lockfile --prefer-offline; pnpm verify:fast'`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-59
- [x] `p3_ac4` command - old payment-specific authority/proof wire names are gone from generic contracts/schemas/fixtures
  - Command: `node scripts/check-runtime-cutover-legacy.mjs --final`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-60

## Phase 4: Green all gates and close

Status: completed
Dependencies: none

Objective: prove the finalized tree green end to end.

Changes:
- none
Acceptance:
- [x] `p4_ac1` command - Rust gates
  - Command: `docker volume create runx-cargo-cache >/dev/null && docker volume create runx-rust-target >/dev/null && docker run --rm -v "$PWD":/work -v runx-cargo-cache:/usr/local/cargo -v runx-rust-target:/cargo-target -w /work rust:1.95-bookworm bash -c 'set -euo pipefail; export PATH=/usr/local/cargo/bin:/usr/local/rustup/toolchains/1.95.0-aarch64-unknown-linux-gnu/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin; apt-get update >/dev/null && apt-get install -y pkg-config libssl-dev ca-certificates curl tar nodejs >/dev/null; if ! command -v cargo-nextest >/dev/null 2>&1 || ! cargo nextest --version | grep -q "0.9.137"; then rm -f /usr/local/cargo/bin/cargo-nextest; curl -LsSf https://get.nexte.st/0.9.137/linux-arm | tar zxf - -C /usr/local/cargo/bin; fi; rustup component add rustfmt clippy >/dev/null; export CARGO_TARGET_DIR=/cargo-target; cargo fmt --manifest-path crates/Cargo.toml --all --check && cargo clippy --manifest-path crates/Cargo.toml --workspace --all-targets --all-features -- -D warnings && cargo nextest run --manifest-path crates/Cargo.toml --workspace --all-features && cargo test --manifest-path crates/Cargo.toml --workspace --all-features --doc'`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-21
- [x] `p4_ac2` command - TS + fixtures
  - Command: `docker volume create runx-cargo-cache >/dev/null && docker volume create runx-rust-target >/dev/null && docker volume create runx-pnpm-store >/dev/null && docker run --rm -v "$PWD":/src:ro -v runx-cargo-cache:/usr/local/cargo -v runx-rust-target:/cargo-target -v runx-pnpm-store:/pnpm-store -w /tmp rust:1.95-bookworm bash -c 'set -euo pipefail; export PATH=/usr/local/cargo/bin:/usr/local/rustup/toolchains/1.95.0-aarch64-unknown-linux-gnu/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin; apt-get update >/dev/null && apt-get install -y pkg-config libssl-dev ca-certificates curl tar gnupg >/dev/null; curl -fsSL https://deb.nodesource.com/setup_20.x | bash - >/dev/null; apt-get install -y nodejs >/dev/null; npm install -g pnpm@10.18.2 >/dev/null; node --version; pnpm --version; mkdir -p /work; tar -C /src --exclude=./node_modules --exclude=./crates/target --exclude=./.git -cf - . | tar -C /work -xf -; cd /work; rm -rf crates/target && ln -s /cargo-target crates/target; export CARGO_TARGET_DIR=/cargo-target; pnpm config set store-dir /pnpm-store; pnpm install --frozen-lockfile --prefer-offline; pnpm verify:fast && pnpm fixtures:harness:check'`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-22
- [x] `p4_ac3` command - license boundary
  - Command: `node .scafld/scripts/check-license-edges.mjs --check manifest-complete && node .scafld/scripts/check-license-edges.mjs --check identifiers`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-23
- [x] `p4_ac4` command - deferred items are recorded as future specs/handoffs
  - Command: `node -e "const fs=require('fs');const spec=fs.readFileSync('.scafld/specs/active/runx-cutover-finalization-v1.md','utf8');const handoff=fs.readFileSync('../plans/codex-handoff-cutover-finalization.md','utf8');const governed=fs.readFileSync('../plans/governed-execution-layer.md','utf8');const ok=spec.includes('Out of scope (future / separate specs)')&&spec.includes('EffectFinalityReceipt')&&spec.includes('Real rails (x402/Stripe/MPP)')&&spec.includes('cloud-to-kernel bridge')&&spec.includes('OpenAPI front')&&handoff.includes('Out of scope (future): real rails')&&handoff.includes('cloud bridge')&&handoff.includes('OpenAPI front')&&governed.includes('Harden the OpenAPI external-adapter example')&&governed.includes('cloud-to-kernel bridge');if(!ok)process.exit(1)"`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-24

## Rollback

- Phases 1, 2, 4 are local and revert cleanly.
- Phase 3 is the only contract-touching step; keep it a single revertible commit.
  It is an intentional hard cutover, so fixture/schema churn is expected.

## Resulting shape (after this spec)

- Generic crates (`runx-contracts`/`runx-runtime`/`runx-core`) are payment-free and
  domain-free under one gate; payment is one opaque effect family in `runx-pay`.
- One canonical tool `source_hash` (or two pinned by a test); one canonical
  story-milestone vocabulary.
- Docs accurate at HEAD; the effect-kernel spec's deferred Phase 4 is closed.
- All nine CI gates green; no skill or contract surface broken.
- Remaining runx work is explicitly future/separate: real rails + the
  cloud-to-kernel bridge, finality-receipt emission, and the OpenAPI front.

## Review

Status: completed
Verdict: pass
Mode: verify
Provider: codex
Output: codex.output_file
Summary: Deep verify review found no completion-blocking issues. I attacked the generic-contract cutover, final domain-free guard, source-hash parity, story vocabulary de-duplication, effect supervisor generalization, caller compatibility, docs/schema mirror consistency, cleanup, and cloud protocol boundary. Available validation commands passed: `node scripts/check-runtime-cutover-legacy.mjs --final` and `node ../cloud/scripts/check-workspace-boundary.mjs`. Direct `./bin/scafld status` could not run because `./bin/scafld` is absent in this checkout.

Attack log:
- `.scafld/prompts/review.md`: review_prompt -> clean (Loaded `.scafld/prompts/review.md` and followed adversarial review instructions.)
- `./bin/scafld status runx-cutover-finalization-v1 --json`: scafld_status -> skipped (Skipped direct scafld status because `./bin/scafld` is absent in this checkout; continued with packet evidence, code inspection, and available validation commands.)
- `workspace dirty state`: workspace_state -> clean (`git status --short` showed the expected broad dirty state from task plus ambient drift. Git emitted macOS temp-cache warnings due read-only sandbox but returned status.)
- `crates/runx-contracts/src/authority.rs; crates/runx-contracts/src/reference.rs`: generic_contracts -> clean (Read `authority.rs` and `reference.rs`; payment-specific authority/proof structs are gone and the remaining seam is family-keyed effect limits/guards plus `EffectEvidence`/`EffectFinality`.)
- `scripts/check-runtime-cutover-legacy.mjs --final`: legacy_domain_gate -> clean (Read and ran the final legacy gate. `checkFinalRustKernelDomainFree` includes `crates/runx-contracts/src`; `node scripts/check-runtime-cutover-legacy.mjs --final` passed.)
- `tools/thread/story.ts; tools/outbox/story.ts`: story_vocabulary_dry -> clean (`tools/thread/story.ts` now re-exports canonical milestone symbols from `tools/outbox/story.ts`; no duplicate milestone vocabulary remains there.)
- `packages/cli/src/authoring-utils.ts; packages/cli/src/authoring-utils.test.ts; crates/runx-runtime/src/tool_catalogs/build.rs`: source_hash_parity -> clean (Compared Rust and TS hash implementations and the new parity test. Scanner escape handling, query/hash stripping, candidate ordering, path hashing, and no-source sentinel align for the covered behavior.)
- `crates/runx-runtime/src/execution/runner/authority.rs; crates/runx-runtime/src/execution/target_runner/pull_request.rs`: caller_compatibility -> clean (Checked effect receipt request helper and target-runner import cleanup. The helper preserves the prior request fields for finalize/persist; pull_request imports directly from `super::`.)
- `crates/runx-runtime/src/receipts/seal.rs`: test_cleanup -> clean (The prior write-only `_message` test field is gone; the error string is now stored and used by `Display`.)
- `crates/runx-pay/src/runtime.rs`: effect_supervisor_generalization -> clean (The supervisor trait now accepts a generic family and opaque payload, returns generic evidence, and payment interpretation stays inside `runx-pay`.)
- `docs/rust-kernel-architecture.md; crates/README.md; crates/runx-runtime/README.md; ../plans/governed-execution-layer.md`: doc_reconciliation -> clean (Checked refreshed feature lists and governed plan anchors. Docs include `cli-tool, mcp, mcp-http-server, a2a, agent, catalog, external-adapter, http` and flag `a2a` as not enabled in runx-cli.)
- `packages/contracts/src; schemas; crates/runx-contracts/src`: contracts_mirror -> clean (Checked TS generated contract hook and retired wire-name scan. No retired payment-specific generic contract names were found in contracts/schemas/task code.)
- `crates/runx-runtime/src/payment; crates/runx-core/src/policy/payment_authority`: empty_dir_cleanup -> clean (Neither requested leftover directory exists.)
- `../cloud/scripts/check-workspace-boundary.mjs; ../cloud package sources`: cloud_protocol_boundary -> clean (Read and ran cloud boundary enforcement. `node ../cloud/scripts/check-workspace-boundary.mjs` passed; production cloud sources use `@runx/protocol` and the boundary guard rejects retired `@runxhq/core`.)

Findings:
- none

