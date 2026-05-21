import type {
  ResolutionRequestContract as ResolutionRequest,
  ResolutionResponseContract as ResolutionResponse,
} from "@runxhq/contracts";

type AgentActResolutionRequest = Extract<ResolutionRequest, { readonly kind: "agent_act" }>;

export interface CliAgentRuntime {
  readonly label: string;
  readonly resolve: (request: AgentActResolutionRequest) => Promise<ResolutionResponse>;
}

export async function loadCliAgentRuntime(
  env: NodeJS.ProcessEnv = process.env,
): Promise<CliAgentRuntime | undefined> {
  void env;
  return undefined;
}
