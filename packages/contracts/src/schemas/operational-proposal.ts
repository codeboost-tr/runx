import {
  type DeepReadonly,
  generatedSchema,
  validateContractSchema,
} from "../internal.js";
import type { ReferenceContract, ReferenceLinkContract } from "./spine.js";

export const operationalProposalSchemaVersion = "runx.operational_proposal.v1" as const;

export type OperationalProposalRedactionStatusContract =
  | "redacted"
  | "summary_only"
  | "blocked";

export type OperationalProposalRecommendedActionContract = DeepReadonly<{
  action_intent: string;
  summary: string;
  mutating: boolean;
  target_refs?: readonly ReferenceContract[];
}>;

export type OperationalProposalIdempotencyContract = DeepReadonly<{
  key: string;
  fingerprint: string;
}>;

export type OperationalProposalAuthorityContract = DeepReadonly<{
  proposal_only: true;
  mutation_authority_granted: false;
  publication_authority_granted: false;
  final_decision_authority_granted: false;
  notes?: readonly string[];
}>;

export type OperationalProposalHumanGateContract = DeepReadonly<{
  gate_id: string;
  gate_kind: string;
  required: boolean;
  decision: string;
  reason: string;
}>;

export type OperationalProposalOutcomeContract = DeepReadonly<{
  observed: boolean;
  status: string;
  summary: string;
  observed_at?: string;
  refs?: readonly ReferenceContract[];
}>;

export type OperationalProposalContract = DeepReadonly<{
  schema: typeof operationalProposalSchemaVersion;
  proposal_id: string;
  proposal_kind: string;
  source_event_id: string;
  idempotency: OperationalProposalIdempotencyContract;
  source_ref: ReferenceContract;
  source_thread_ref?: ReferenceContract;
  hydrated_context_ref: ReferenceContract;
  redaction_status: OperationalProposalRedactionStatusContract;
  decision_summary: string;
  rationale: string;
  recommended_actions?: readonly OperationalProposalRecommendedActionContract[];
  evidence_refs?: readonly ReferenceContract[];
  artifact_refs?: readonly ReferenceContract[];
  receipt_refs?: readonly ReferenceContract[];
  story_refs?: readonly ReferenceContract[];
  result_refs?: readonly ReferenceLinkContract[];
  publication_refs?: readonly ReferenceLinkContract[];
  owner_route_id: string;
  confidence: number;
  risks?: readonly string[];
  caveats?: readonly string[];
  missing_context?: readonly string[];
  authority: OperationalProposalAuthorityContract;
  human_gates?: readonly OperationalProposalHumanGateContract[];
  allowed_next_actions?: readonly string[];
  final_outcome?: OperationalProposalOutcomeContract;
  public_summary: string;
  extensions?: DeepReadonly<Record<string, unknown>>;
}>;

export const operationalProposalSchema = generatedSchema<OperationalProposalContract>(
  "operational-proposal.schema.json",
);

export function validateOperationalProposalContract(
  value: unknown,
  label = "operational_proposal",
): OperationalProposalContract {
  return validateContractSchema(operationalProposalSchema, value, label) as OperationalProposalContract;
}
