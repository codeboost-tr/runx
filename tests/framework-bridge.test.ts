import { afterEach, describe, expect, it } from "vitest";

import { createStructuredCaller } from "@runxhq/runtime-local/sdk";
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

describe("host bridge", () => {
  it("pauses on unresolved work and resumes the same run through the shared bridge", async () => {
    const harness = await createHostHarness();
    cleanups.push(harness.cleanup);

    const needsAgent = await harness.bridge.run({
      skillPath: "fixtures/skills/echo",
    });

    expect(needsAgent.status).toBe("needs_agent");
    if (needsAgent.status !== "needs_agent") {
      return;
    }
    expect(needsAgent.requests[0]).toMatchObject({
      kind: "input",
    });

    const continued = await harness.bridge.resume(needsAgent.runId, {
      skillPath: "fixtures/skills/echo",
      resolver: ({ request }) => {
        if (request.kind !== "input") {
          return undefined;
        }
        return { message: "from-host-bridge" };
      },
    });

    expect(continued).toMatchObject({
      status: "completed",
      skillName: "echo",
      output: "from-host-bridge",
    });
  }, 20_000);

  it("falls back to an upstream caller when the bridge resolver does not answer", async () => {
    const harness = await createHostHarness();
    cleanups.push(harness.cleanup);
    const caller = createStructuredCaller({
      answers: {
        message: "from-upstream-caller",
      },
    });

    const result = await harness.bridge.run({
      skillPath: "fixtures/skills/echo",
      caller,
    });

    expect(result).toMatchObject({
      status: "completed",
      output: "from-upstream-caller",
    });
    expect(caller.trace.resolutions).toHaveLength(1);
  });
});
