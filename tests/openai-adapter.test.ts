import { afterEach, describe, expect, it } from "vitest";

import { createOpenAiHostAdapter } from "@runxhq/host-adapters";
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

describe("OpenAI host adapter", () => {
  it("wraps needsAgent and continued runs in an OpenAI-style tool response", async () => {
    const harness = await createHostHarness();
    cleanups.push(harness.cleanup);
    const adapter = createOpenAiHostAdapter(harness.bridge);

    const needsAgent = await adapter.run({
      skillPath: "fixtures/skills/echo",
    });

    expect(needsAgent.role).toBe("tool");
    expect(needsAgent.structuredContent.runx.status).toBe("needs_agent");
    if (needsAgent.structuredContent.runx.status !== "needs_agent") {
      return;
    }

    const continued = await adapter.resume(needsAgent.structuredContent.runx.runId, {
      skillPath: "fixtures/skills/echo",
      resolver: ({ request }) => (request.kind === "input" ? { message: "from-openai-host-adapter" } : undefined),
    });

    expect(continued.role).toBe("tool");
    expect(continued.structuredContent.runx).toMatchObject({
      status: "completed",
      output: "from-openai-host-adapter",
    });
  }, 20_000);
});
