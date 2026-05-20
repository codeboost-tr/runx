import { afterEach, describe, expect, it } from "vitest";

import { createAnthropicHostAdapter } from "@runxhq/host-adapters";
import { createHostHarness } from "./host-protocol-test-utils.js";

const cleanups: Array<() => Promise<void>> = [];

afterEach(async () => {
  while (cleanups.length > 0) {
    const cleanup = cleanups.pop();
    if (cleanup) {
      await cleanup();
    }
  }
});

describe("Anthropic host adapter", () => {
  it("wraps needsAgent and continued runs in an Anthropic-style response", async () => {
    const harness = await createHostHarness();
    cleanups.push(harness.cleanup);
    const adapter = createAnthropicHostAdapter(harness.bridge);

    const needsAgent = await adapter.run({
      skillPath: "fixtures/skills/echo",
    });

    expect(needsAgent.metadata.runx.status).toBe("needs_agent");
    if (needsAgent.metadata.runx.status !== "needs_agent") {
      return;
    }

    const continued = await adapter.resume(needsAgent.metadata.runx.runId, {
      skillPath: "fixtures/skills/echo",
      resolver: ({ request }) => (request.kind === "input" ? { message: "from-anthropic-host-adapter" } : undefined),
    });

    expect(continued.metadata.runx).toMatchObject({
      status: "completed",
      output: "from-anthropic-host-adapter",
    });
  }, 20_000);
});
