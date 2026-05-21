---
spec_version: '2.0'
task_id: external-adapter-runtime-wiring-v1
created: '2026-05-22T00:00:00+10:00'
updated: '2026-05-22T00:55:00+10:00'
status: active
harden_status: not_run
size: medium
risk_level: high
---

# External adapter runtime wiring

## Current State

Status: complete
Current phase: runtime wiring slice complete
Next: keep final manifest discovery, credential material delivery, and
host-resolution routing blocked under `external-adapter-plugin-protocol-v1`.
Reason: `external-adapter-plugin-protocol-v1` owns the broad protocol, SDK,
and runtime-local sunset story. This focused spec owns only the Rust runtime
adapter-selection slice for `source_type: external-adapter`.
Blockers: durable manifest discovery semantics are not frozen. This slice may
only support an explicit inline manifest or injected manifest resolver; registry
lookup, package-relative manifest paths, remote manifests, credential delivery,
host-resolution routing, and provider-specific adapter behavior remain blocked
under the broader protocol spec.
Allowed follow-up command: `scafld harden external-adapter-runtime-wiring-v1`
Latest runner update: 2026-05-22T00:55:00+10:00 added the feature-gated
`ExternalAdapterSkillAdapter`, inline-manifest resolver, injectable manifest
resolver/supervisor traits, graph routing coverage, fail-closed tests, and
reran the focused test after cleaning unrelated harness warnings.
Review gate: implementation_ready

## Summary

Expose the existing feature-gated external adapter process supervisor as a
usable Rust runtime adapter path. The adapter path must accept
`source_type: external-adapter`, build the contract invocation frame under Rust
authority, call the supervisor, and convert accepted adapter observations into
normal `SkillOutput` values. It must not add provider-specific integration code
to the runtime kernel.

This spec is intentionally narrower than
`external-adapter-plugin-protocol-v1`. It is a wiring slice, not the final
manifest discovery, credential, host-resolution, or SDK story.

## Context

Primary owner spec:
- `.scafld/specs/active/external-adapter-plugin-protocol-v1.md`

Owned touchpoints:
- `crates/runx-runtime/src/adapters/external_adapter.rs`
- `crates/runx-runtime/tests/external_adapter.rs`
- minimal runtime adapter-selection code when needed

Out of scope:
- x402 payment runtime, tests, and fixtures.
- canonical-json, core, and cloud files.
- provider-specific GitHub, Slack, Sentry, or hosted adapter logic.
- TypeScript helper SDKs.
- final manifest registry/discovery semantics.

## Objectives

- Add a feature-gated `SkillAdapter` facade for `external-adapter`.
- Keep the supervisor authoritative: the facade only builds contract frames,
  resolves manifests, calls the supervisor, and maps observations into runtime
  outputs.
- Preserve fail-closed behavior when the source type is unsupported, the
  manifest is absent or malformed, the response identity mismatches, the
  adapter crashes, or the response cannot safely map to runtime output.
- Prove a graph or skill invocation with `source_type: external-adapter`
  reaches the supervisor.

## Acceptance

Profile: strict

Definition of done:
- [x] `dod1` Runtime exposes a feature-gated external adapter `SkillAdapter`
  path for `source_type: external-adapter`.
- [x] `dod2` The path builds `ExternalAdapterInvocation` from runtime
  `SkillInvocation` without importing provider-specific logic.
- [x] `dod3` Manifest discovery remains explicit. Inline manifests or injected
  resolvers are allowed for this slice; implicit package/registry lookup is
  blocked until `external-adapter-plugin-protocol-v1` settles it.
- [x] `dod4` Tests prove a graph/skill invocation reaches the supervisor and
  fail-closed behavior is preserved.

Validation:
- [x] `v1` Focused Rust tests pass.
  - Command:
    `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features external-adapter --test external_adapter -- --nocapture`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-22T00:55:00+10:00 passed 9 tests, including
    `external_adapter_graph_invocation_reaches_process_supervisor`,
    `external_adapter_skill_adapter_fails_closed_without_inline_manifest`, and
    `external_adapter_skill_adapter_preserves_supervisor_fail_closed_response_mismatch`.
- [x] `v2` Focused spec validates.
  - Command: `scafld validate external-adapter-runtime-wiring-v1 --json`
  - Expected kind: `exit_code_zero`
  - Status: passed
  - Evidence: 2026-05-22T00:42:00+10:00 returned
    `{"ok":true,"command":"validate","result":{"task_id":"external-adapter-runtime-wiring-v1","path":"/Users/kam/dev/runx/runx/oss/.scafld/specs/active/external-adapter-runtime-wiring-v1.md","valid":true,"errors":null}}`.

## Design Constraints

- Feature-gated code must not be reachable without `features =
  ["external-adapter"]`.
- Runtime-local or `@runxhq/adapters` must not be used as a fallback.
- The Rust runtime may parse an explicit manifest and frame data, but must not
  understand provider APIs.
- If manifest lookup requires a new package or registry convention, this spec
  must record that as a blocker rather than inventing a broad discovery system.

## Blocker Evidence

The current parser/runtime `SkillSource` retains arbitrary raw source metadata,
but it has no typed external-adapter manifest location field. This slice can
therefore prove wiring with an inline manifest or caller-provided resolver, but
cannot claim final package-relative or registry-backed manifest discovery.

## Rollback

Remove the feature-gated adapter facade and keep the process supervisor as an
explicit API if runtime selection cannot preserve Rust authority or requires
provider-specific logic.
