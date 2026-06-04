#!/bin/sh
set -eu

INPUT=$(cat)
IDEMPOTENCY_STATUS="${2:-created}"

if printf "%s" "$INPUT" | grep -q '"fetch_id"'; then
  cat <<'JSON'
{
  "schema": "runx.thread_outbox_provider.observation.v1",
  "protocol_version": "runx.thread_outbox_provider.v1",
  "observation_id": "thread_obs_fetch_123",
  "adapter_id": "thread-provider.github",
  "provider": "github",
  "operation": "fetch",
  "request_id": "thread_fetch_123",
  "status": "accepted",
  "idempotency": {
    "key": "thread-outbox:github:runxhq/runx#77:fetch",
    "status": "replayed"
  },
  "provider_locator": {
    "provider": "github",
    "locator": "runxhq/runx#77/comment-1001"
  },
  "provider_event_id_hash": "sha256:github-comment-1001",
  "readback_summary": {
    "item_count": 1,
    "cursor": "cursor-2",
    "latest_provider_event_id_hash": "sha256:github-comment-1001"
  },
  "observed_at": "2026-05-22T00:00:03Z"
}
JSON
  exit 0
fi

cat <<JSON
{
  "schema": "runx.thread_outbox_provider.observation.v1",
  "protocol_version": "runx.thread_outbox_provider.v1",
  "observation_id": "thread_obs_123",
  "adapter_id": "thread-provider.github",
  "provider": "github",
  "operation": "push",
  "request_id": "thread_push_123",
  "status": "accepted",
  "idempotency": {
    "key": "thread-outbox:github:runxhq/runx#77:outbox_entry_123",
    "status": "${IDEMPOTENCY_STATUS}"
  },
  "provider_locator": {
    "provider": "github",
    "locator": "runxhq/runx#77/comment-1001",
    "provider_ref": {
      "type": "external_url",
      "uri": "https://github.com/runxhq/runx/issues/77#issuecomment-1001",
      "provider": "github"
    }
  },
  "provider_event_id_hash": "sha256:github-comment-1001",
  "readback_summary": {
    "item_count": 1,
    "cursor": "cursor-2",
    "latest_provider_event_id_hash": "sha256:github-comment-1001"
  },
  "redaction_refs": [
    {
      "type": "redaction_policy",
      "uri": "runx:redaction_policy:provider-output"
    }
  ],
  "observed_at": "2026-05-22T00:00:02Z"
}
JSON
