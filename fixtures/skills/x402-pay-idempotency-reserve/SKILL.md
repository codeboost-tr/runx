---
name: pay-reserve
description: Deterministically reserve the x402 idempotency fixture spend.
source:
  type: cli-tool
  command: sh
  args:
    - ./run.sh
  timeout_seconds: 10
  sandbox:
    profile: readonly
    cwd_policy: skill-directory
    env_allowlist:
      - RUNX_X402_GRAPH_NAME
      - RUNX_X402_IDEMPOTENCY_KEY
inputs: {}
---

Emit a deterministic reserved payment authority for the x402 idempotency fixture.
