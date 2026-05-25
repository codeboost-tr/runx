---
spec_version: '2.0'
task_id: process-credential-delivery-hardening-v1
created: '2026-05-25T17:51:35+10:00'
updated: '2026-05-25T17:51:35+10:00'
status: draft
harden_status: not_run
size: medium
risk_level: high
---

# process-credential-delivery-hardening-v1

## Current State

Status: draft
Current phase: ready for execution
Next: replace or strictly constrain env-based credential delivery for
supervised process adapters.
Reason: `cli-tool` now rejects process-env credential delivery before spawn, but
MCP, external adapters, and outbox providers still have process boundaries where
credentials may be delivered via environment variables.
Blockers: none.
Allowed follow-up command: `scafld exec process-credential-delivery-hardening-v1`
Latest runner update: 2026-05-25T17:51:35+10:00
Review gate: not_started

## Summary

Credentials crossing supervised process boundaries must be brokered by opaque
references, scoped files, or a runtime-owned descriptor channel. Raw secrets
must not be ambient child process environment. Redaction remains defense in
depth, not containment.

## Scope

In scope:
- MCP adapter credential delivery.
- External adapter credential delivery.
- Outbox provider credential delivery.
- Receipt-safe observation and redaction metadata for the selected channel.

Out of scope:
- Provider-specific OAuth flows.
- `cli-tool` env secret rejection, already implemented.

## Acceptance

Profile: strict

Definition of done:
- [ ] `dod1` MCP process delivery does not expose raw secrets through ambient
  child environment.
- [ ] `dod2` External adapter process delivery does not expose raw secrets
  through ambient child environment.
- [ ] `dod3` Outbox provider process delivery does not expose raw secrets
  through ambient child environment.
- [ ] `dod4` Receipts record credential handle/observation metadata without
  leaking secret material.

Validation:
- [ ] `v1` MCP process credential delivery tests
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --features mcp --test mcp_server`
  - Expected kind: `exit_code_zero`
  - Timeout seconds: none
  - Result: none
  - Status: pending
  - Evidence: MCP process delivery cannot retain raw secrets in ambient child env
  - Source event: none
  - Last attempt: none
  - Checked at: none
- [ ] `v2` external-adapter credential delivery tests
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test external_adapter`
  - Expected kind: `exit_code_zero`
  - Timeout seconds: none
  - Result: none
  - Status: pending
  - Evidence: external adapter process delivery cannot retain raw secrets in ambient child env
  - Source event: none
  - Last attempt: none
  - Checked at: none
- [ ] `v3` credential-delivery contract tests
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test credential_delivery`
  - Expected kind: `exit_code_zero`
  - Timeout seconds: none
  - Result: none
  - Status: pending
  - Evidence: public observations carry opaque handles/hashes/refs without secret material
  - Source event: none
  - Last attempt: none
  - Checked at: none
- [ ] `v4` focused process-env delivery grep review
  - Command: `rg -n "CredentialDelivery::ProcessEnv|secret_env\\(|\\.envs\\(secret_env|process_env" crates/runx-runtime/src crates/runx-runtime/tests`
  - Expected kind: `reviewed_output`
  - Timeout seconds: none
  - Result: none
  - Status: pending
  - Evidence: no supervised process adapter keeps raw secrets in long-lived child environment
  - Source event: none
  - Last attempt: none
  - Checked at: none

## Review

Reject any patch that treats substring redaction as credential containment or
allows a raw secret to remain in a long-lived child environment.
