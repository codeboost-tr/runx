import { describe, expect, it } from "vitest";

import {
  createAnthropicHostAdapter,
  createCrewAiHostAdapter,
  createLangChainHostAdapter,
  createOpenAiHostAdapter,
  createVercelAiHostAdapter,
  type HostBridge,
  type HostRunResult,
  type HostRunState,
} from "./index.js";

function fakeBridge(result: HostRunResult): HostBridge {
  return {
    run: async () => result,
    resume: async () => result,
    inspect: async () => result as HostRunState,
  };
}

describe("host host adapters", () => {
  const needsAgent: HostRunResult = {
    status: "needs_agent",
    skillName: "echo",
    runId: "rx_paused",
    requests: [],
    events: [],
  };

  it("wraps OpenAI tool responses", async () => {
    const response = await createOpenAiHostAdapter(fakeBridge(needsAgent)).run({ skillPath: "unused" });
    expect(response).toMatchObject({
      role: "tool",
      structuredContent: {
        runx: {
          status: "needs_agent",
          runId: "rx_paused",
        },
      },
    });
  });

  it("wraps Anthropic responses", async () => {
    const response = await createAnthropicHostAdapter(fakeBridge(needsAgent)).run({ skillPath: "unused" });
    expect(response.metadata.runx.status).toBe("needs_agent");
  });

  it("wraps Vercel AI SDK responses", async () => {
    const response = await createVercelAiHostAdapter(fakeBridge(needsAgent)).run({ skillPath: "unused" });
    expect(response.data.runx.status).toBe("needs_agent");
  });

  it("wraps LangChain responses", async () => {
    const response = await createLangChainHostAdapter(fakeBridge(needsAgent)).run({ skillPath: "unused" });
    expect(response.additional_kwargs.runx.status).toBe("needs_agent");
  });

  it("wraps CrewAI responses", async () => {
    const response = await createCrewAiHostAdapter(fakeBridge(needsAgent)).run({ skillPath: "unused" });
    expect(response.json_dict.runx.status).toBe("needs_agent");
  });
});
