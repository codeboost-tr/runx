---
name: pay-fulfill-rail
description: Deterministically fulfill or partially mutate the x402 idempotency fixture rail spend.
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
      - RUNX_PAYMENT_RAIL_COUNT_PATH
      - RUNX_X402_IDEMPOTENCY_KEY
      - RUNX_X402_RAIL_MODE
inputs: {}
---

Emit deterministic mock rail packets for x402 idempotency replay and recovery.
