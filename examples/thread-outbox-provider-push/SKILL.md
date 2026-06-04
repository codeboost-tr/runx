---
name: thread-outbox-provider-push
description: Publish a fixture thread outbox entry through the Rust thread-outbox-provider front.
source:
  type: thread-outbox-provider
  thread_outbox_provider:
    operation: push
    manifest_path: manifest.json
    push_path: push.json
---
# Thread Outbox Provider Push

Publishes the fixture outbox entry through the governed Rust provider front.
