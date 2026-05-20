import { afterEach, describe, expect, it } from "vitest";

import { createVercelAiHostAdapter } from "@runxhq/host-adapters";
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

describe("Vercel AI host adapter", () => {
  it("wraps needsAgent and continued runs in a Vercel AI-style response", async () => {
    const harness = await createHostHarness();
    cleanups.push(harness.cleanup);
    const adapter = createVercelAiHostAdapter(harness.bridge);

    const needsAgent = await adapter.run({
      skillPath: "fixtures/skills/echo",
    });

    expect(needsAgent.data.runx.status).toBe("needs_agent");
    if (needsAgent.data.runx.status !== "needs_agent") {
      return;
    }

    const continued = await adapter.resume(needsAgent.data.runx.runId, {
      skillPath: "fixtures/skills/echo",
      resolver: ({ request }) => (request.kind === "input" ? { message: "from-vercel-ai-host-adapter" } : undefined),
    });

    expect(continued.data.runx).toMatchObject({
      status: "completed",
      output: "from-vercel-ai-host-adapter",
    });
  }, 20_000);
});
