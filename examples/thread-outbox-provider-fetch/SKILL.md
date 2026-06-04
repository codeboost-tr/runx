---
name: thread-outbox-provider-fetch
description: Fetch fixture thread readback through the Rust thread-outbox-provider front.
source:
  type: thread-outbox-provider
  thread_outbox_provider:
    operation: fetch
    manifest_path: manifest.json
    fetch_path: fetch.json
---
# Thread Outbox Provider Fetch

Reads back the fixture provider thread through the governed Rust provider front.
