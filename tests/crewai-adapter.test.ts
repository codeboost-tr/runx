import { afterEach, describe, expect, it } from "vitest";

import { createCrewAiHostAdapter } from "@runxhq/host-adapters";
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

describe("CrewAI host adapter", () => {
  it("wraps needsAgent and continued runs in a CrewAI-style response", async () => {
    const harness = await createHostHarness();
    cleanups.push(harness.cleanup);
    const adapter = createCrewAiHostAdapter(harness.bridge);

    const needsAgent = await adapter.run({
      skillPath: "fixtures/skills/echo",
    });

    expect(needsAgent.json_dict.runx.status).toBe("needs_agent");
    if (needsAgent.json_dict.runx.status !== "needs_agent") {
      return;
    }

    const continued = await adapter.resume(needsAgent.json_dict.runx.runId, {
      skillPath: "fixtures/skills/echo",
      resolver: ({ request }) => (request.kind === "input" ? { message: "from-crewai-host-adapter" } : undefined),
    });

    expect(continued.json_dict.runx).toMatchObject({
      status: "completed",
      output: "from-crewai-host-adapter",
    });
  }, 20_000);
});
