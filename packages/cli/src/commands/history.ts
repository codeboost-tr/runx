import { resolvePathFromUserInput } from "@runxhq/core/config";

import { runNativeRunxJson } from "../native-runx.js";
import { renderKeyValue, relativeTime, shortId, statusIcon, theme } from "../ui.js";

export interface InspectCommandArgs {
  readonly receiptId: string;
  readonly receiptDir?: string;
}

export interface HistoryCommandArgs {
  readonly receiptDir?: string;
  readonly historyQuery?: string;
  readonly historySkill?: string;
  readonly historyStatus?: string;
  readonly historySource?: string;
  readonly historyActor?: string;
  readonly historyArtifactType?: string;
  readonly historySince?: string;
  readonly historyUntil?: string;
}

export interface ReplayCommandArgs {
  readonly replayRef: string;
  readonly receiptDir?: string;
}

export interface DiffCommandArgs {
  readonly diffLeft: string;
  readonly diffRight: string;
  readonly receiptDir?: string;
}

export interface LocalReceiptSummary {
  readonly id: string;
  readonly kind: string;
  readonly name: string;
  readonly status: string;
  readonly sourceType?: string;
  readonly disposition?: string;
  readonly outcomeState?: string;
  readonly startedAt?: string;
  readonly completedAt?: string;
  readonly actors?: readonly string[];
  readonly artifactTypes?: readonly string[];
  readonly runnerProvider?: string;
  readonly approval?: {
    readonly decision?: string;
    readonly gateType?: string;
  };
  readonly lineage?: {
    readonly kind: string;
    readonly sourceRunId: string;
  };
  readonly verification?: {
    readonly status?: string;
    readonly reason?: string;
  };
  readonly ledgerVerification?: {
    readonly status?: string;
    readonly reason?: string;
  };
  readonly harnessId?: string;
  readonly harnessState?: string;
  readonly harnessSealSummary?: string;
}

export interface PausedRunSummary {
  readonly id: string;
  readonly kind: string;
  readonly name: string;
  readonly status: string;
  readonly selectedRunner?: string;
  readonly stepIds: readonly string[];
  readonly stepLabels: readonly string[];
  readonly ledgerVerification?: {
    readonly status?: string;
    readonly reason?: string;
  };
}

export type InspectLocalRunResult =
  | { readonly kind: "paused"; readonly summary: PausedRunSummary }
  | { readonly kind: "receipt"; readonly summary: LocalReceiptSummary };

export interface RunSummaryDiff {
  readonly changed: boolean;
  readonly left: { readonly id: string; readonly name: string };
  readonly right: { readonly id: string; readonly name: string };
  readonly fields: Readonly<Record<string, { readonly left: unknown; readonly right: unknown }>>;
  readonly actors: { readonly added: readonly string[]; readonly removed: readonly string[] };
  readonly artifactTypes: { readonly added: readonly string[]; readonly removed: readonly string[] };
}

interface ReplaySeed {
  readonly skillPath: string;
  readonly inputs: Readonly<Record<string, unknown>>;
  readonly selectedRunner?: string;
}

export async function handleInspectCommand(
  parsed: InspectCommandArgs,
  env: NodeJS.ProcessEnv,
): Promise<{ readonly summary: LocalReceiptSummary }> {
  void parsed;
  void env;
  throw new Error("native receipt inspection is not implemented yet; use `runx history --json` for sealed harness receipts.");
}

export async function handleInspectRunCommand(
  parsed: InspectCommandArgs,
  env: NodeJS.ProcessEnv,
): Promise<InspectLocalRunResult> {
  void parsed;
  void env;
  throw new Error("native receipt inspection is not implemented yet; use `runx history --json` for sealed harness receipts.");
}

export async function handleHistoryCommand(
  parsed: HistoryCommandArgs,
  env: NodeJS.ProcessEnv,
): Promise<{ readonly receipts: readonly LocalReceiptSummary[]; readonly pendingRuns: readonly PausedRunSummary[] }> {
  const args = ["history"];
  if (parsed.historyQuery) args.push(parsed.historyQuery);
  pushOptionalFlag(args, "--receipt-dir", parsed.receiptDir ? resolvePathFromUserInput(parsed.receiptDir, env) : undefined);
  pushOptionalFlag(args, "--skill", parsed.historySkill);
  pushOptionalFlag(args, "--status", parsed.historyStatus);
  pushOptionalFlag(args, "--source", parsed.historySource);
  pushOptionalFlag(args, "--actor", parsed.historyActor);
  pushOptionalFlag(args, "--artifact-type", parsed.historyArtifactType);
  pushOptionalFlag(args, "--since", parsed.historySince);
  pushOptionalFlag(args, "--until", parsed.historyUntil);
  args.push("--json");
  return normalizeHistoryProjection(await runNativeRunxJson(args, { env }));
}

export async function handleReplaySeedCommand(
  parsed: ReplayCommandArgs,
  env: NodeJS.ProcessEnv,
): Promise<ReplaySeed> {
  void parsed;
  void env;
  throw new Error("native replay is not implemented yet; rerun the skill with explicit inputs and --answers.");
}

export async function handleDiffCommand(
  parsed: DiffCommandArgs,
  env: NodeJS.ProcessEnv,
): Promise<RunSummaryDiff> {
  void parsed;
  void env;
  throw new Error("native run diff is not implemented yet; compare `runx history --json` outputs.");
}

function normalizeHistoryProjection(value: unknown): {
  readonly receipts: readonly LocalReceiptSummary[];
  readonly pendingRuns: readonly PausedRunSummary[];
} {
  const projection = asRecord(value);
  if (!projection) {
    throw new Error("native runx history returned a non-object payload.");
  }
  return {
    receipts: arrayValue(projection.receipts).map(normalizeHistoryReceipt),
    pendingRuns: arrayValue(projection.pendingRuns).map(normalizePausedRun),
  };
}

function normalizeHistoryReceipt(value: unknown): LocalReceiptSummary {
  const receipt = asRecord(value);
  if (!receipt || typeof receipt.id !== "string" || typeof receipt.name !== "string" || typeof receipt.status !== "string") {
    throw new Error("native runx history returned an invalid receipt entry.");
  }
  const verification = asRecord(receipt.verification);
  return {
    id: receipt.id,
    kind: stringValue(receipt.source_type) ?? "harness_receipt",
    name: receipt.name,
    status: receipt.status,
    sourceType: stringValue(receipt.source_type),
    startedAt: stringValue(receipt.created_at),
    actors: stringArray(receipt.actors),
    artifactTypes: stringArray(receipt.artifact_types),
    verification: verification ? { status: stringValue(verification.status) } : undefined,
    harnessId: stringValue(receipt.harness_id),
    harnessState: stringValue(receipt.harness_state),
    harnessSealSummary: stringValue(receipt.summary),
  };
}

function normalizePausedRun(value: unknown): PausedRunSummary {
  const run = asRecord(value);
  if (!run || typeof run.id !== "string" || typeof run.name !== "string" || typeof run.kind !== "string" || typeof run.status !== "string") {
    throw new Error("native runx history returned an invalid pending run entry.");
  }
  const ledgerVerification = asRecord(run.ledgerVerification);
  return {
    id: run.id,
    name: run.name,
    kind: run.kind,
    status: run.status === "paused" ? "needs_agent" : run.status,
    selectedRunner: stringValue(run.selectedRunner),
    stepIds: stringArray(run.stepIds),
    stepLabels: stringArray(run.stepLabels),
    ledgerVerification: ledgerVerification
      ? {
          status: stringValue(ledgerVerification.status),
          reason: stringValue(ledgerVerification.reason),
        }
      : undefined,
  };
}

function pushOptionalFlag(args: string[], flag: string, value: string | undefined): void {
  if (value !== undefined && value.length > 0) {
    args.push(flag, value);
  }
}

function asRecord(value: unknown): Readonly<Record<string, unknown>> | undefined {
  return typeof value === "object" && value !== null && !Array.isArray(value)
    ? value as Readonly<Record<string, unknown>>
    : undefined;
}

function arrayValue(value: unknown): readonly unknown[] {
  return Array.isArray(value) ? value : [];
}

function stringValue(value: unknown): string | undefined {
  return typeof value === "string" ? value : undefined;
}

function stringArray(value: unknown): readonly string[] {
  return Array.isArray(value) ? value.filter((entry): entry is string => typeof entry === "string") : [];
}

export function renderReceiptInspection(summary: LocalReceiptSummary, env: NodeJS.ProcessEnv = process.env): string {
  const t = theme(undefined, env);
  const rows: Array<[string, string]> = [
    ["id", summary.id],
    ["kind", summary.kind],
    ["status", summary.status],
  ];
  if (summary.sourceType) rows.push(["source", summary.sourceType]);
  if (summary.disposition) rows.push(["disposition", summary.disposition]);
  if (summary.outcomeState) rows.push(["outcome", summary.outcomeState]);
  if (summary.startedAt) rows.push(["started", relativeTime(summary.startedAt)]);
  if (summary.completedAt) rows.push(["completed", relativeTime(summary.completedAt)]);
  if (summary.actors && summary.actors.length > 0) rows.push(["actors", summary.actors.join(", ")]);
  if (summary.artifactTypes && summary.artifactTypes.length > 0) rows.push(["artifacts", summary.artifactTypes.join(", ")]);
  if (summary.runnerProvider) rows.push(["runner", summary.runnerProvider]);
  if (summary.approval?.decision) rows.push(["approval", `${summary.approval.decision}${summary.approval.gateType ? ` · ${summary.approval.gateType}` : ""}`]);
  if (summary.lineage) rows.push(["lineage", `${summary.lineage.kind} of ${summary.lineage.sourceRunId}`]);
  if (summary.verification) rows.push(["verify", `${summary.verification.status}${summary.verification.reason ? ` (${summary.verification.reason})` : ""}`]);
  if (summary.ledgerVerification) rows.push(["ledger", `${summary.ledgerVerification.status}${summary.ledgerVerification.reason ? ` (${summary.ledgerVerification.reason})` : ""}`]);
  rows.push(["history", "runx history"]);
  rows.push(["json", "runx history --json"]);
  return renderKeyValue(summary.name, summary.status, rows, t);
}

export function renderHistory(
  receipts: readonly LocalReceiptSummary[],
  env: NodeJS.ProcessEnv = process.env,
  query?: string,
  pendingRuns: readonly PausedRunSummary[] = [],
): string {
  const t = theme(undefined, env);
  const totalCount = receipts.length + pendingRuns.length;
  if (totalCount === 0) {
    return query
      ? `\n  ${t.dim}No receipts matched ${t.cyan}${query}${t.reset}${t.dim}.${t.reset}\n  ${t.dim}Try ${t.cyan}runx history${t.reset}${t.dim} to see every local run.${t.reset}\n\n`
      : `\n  ${t.dim}No receipts yet. Try a run first:${t.reset}\n  ${t.cyan}runx skill <skill-dir> --json${t.reset}\n  ${t.cyan}runx list skills${t.reset}\n\n`;
  }
  const now = Date.now();
  const allNames = [...receipts.map((r) => r.name), ...pendingRuns.map((r) => r.name)];
  const nameWidth = Math.min(32, Math.max(...allNames.map((name) => name.length)));
  const lines: string[] = [""];
  const summary = pendingRuns.length > 0
    ? `${receipts.length} receipt(s), ${pendingRuns.length} needs_agent`
    : `${totalCount} receipt(s)`;
  lines.push(`  ${t.bold}history${t.reset}${query ? `  ${t.dim}· ${query}${t.reset}` : ""}  ${t.dim}${summary}${t.reset}`);
  lines.push("");
  for (const pending of pendingRuns) {
    const name = pending.name.padEnd(nameWidth);
    const id = shortId(pending.id);
    const stepLabel = pending.stepLabels[0] ?? pending.stepIds[0] ?? "—";
    lines.push(
      `  ${t.cyan}◇${t.reset}  ${t.bold}${name}${t.reset}  ${t.dim}${pending.status.padEnd(16)}${t.reset}  ${t.dim}${stepLabel.padEnd(10)}${t.reset}  ${t.dim}${"".padEnd(10)}${t.reset}  ${t.dim}${id}${t.reset}`,
    );
  }
  for (const receipt of receipts) {
    const icon = statusIcon(receipt.status, t);
    const name = receipt.name.padEnd(nameWidth);
    const when = receipt.startedAt ? relativeTime(receipt.startedAt, now) : "";
    const source = receipt.sourceType ?? receipt.kind;
    const id = shortId(receipt.id);
    const verification = formatHistoryVerification(receipt);
    lines.push(
      `  ${icon}  ${t.bold}${name}${t.reset}  ${t.dim}${source.padEnd(16)}${t.reset}  ${t.dim}${verification.padEnd(16)}${t.reset}  ${t.dim}${when.padEnd(10)}${t.reset}  ${t.dim}${id}${t.reset}`,
    );
    const harnessStatus = formatHarnessHistoryStatus(receipt);
    if (harnessStatus) {
      lines.push(`     ${t.dim}${harnessStatus}${t.reset}`);
    }
  }
  lines.push("");
  if (pendingRuns.length > 0) {
    lines.push(`  ${t.dim}next${t.reset}  runx skill <same-skill-ref> --run-id <run-id> --answers answers.json  ${t.dim}or${t.reset}  runx history --json`);
  } else {
    lines.push(`  ${t.dim}next${t.reset}  runx history --json`);
  }
  lines.push("");
  return lines.join("\n");
}

function formatHarnessHistoryStatus(receipt: LocalReceiptSummary): string | undefined {
  if (!receipt.harnessState && !receipt.harnessSealSummary && !receipt.harnessId) {
    return undefined;
  }
  const parts = [
    receipt.harnessId ? `harness ${receipt.harnessId}` : "harness",
    receipt.harnessState,
    receipt.harnessSealSummary,
  ].filter((value): value is string => Boolean(value));
  return parts.join(" · ");
}

export function renderPausedRunInspection(
  summary: PausedRunSummary,
  env: NodeJS.ProcessEnv = process.env,
): string {
  const t = theme(undefined, env);
  const rows: Array<[string, string]> = [
    ["id", summary.id],
    ["kind", summary.kind],
    ["status", summary.status],
  ];
  if (summary.selectedRunner) rows.push(["runner", summary.selectedRunner]);
  if (summary.stepIds.length > 0) rows.push(["step", summary.stepIds.join(", ")]);
  if (summary.stepLabels.length > 0) rows.push(["label", summary.stepLabels.join(", ")]);
  if (summary.ledgerVerification) rows.push(["ledger", `${summary.ledgerVerification.status}${summary.ledgerVerification.reason ? ` (${summary.ledgerVerification.reason})` : ""}`]);
  rows.push(["continue", `runx skill <same-skill-ref> --run-id ${summary.id} --answers answers.json`]);
  rows.push(["json", "runx history --json"]);
  return renderKeyValue(summary.name, summary.status, rows, t);
}

export function renderRunDiff(diff: RunSummaryDiff, env: NodeJS.ProcessEnv = process.env): string {
  const t = theme(undefined, env);
  const lines: string[] = [""];
  lines.push(`  ${t.bold}diff${t.reset}  ${t.dim}${shortId(diff.left.id)} -> ${shortId(diff.right.id)}${t.reset}`);
  lines.push(`  ${t.dim}${diff.left.name}${t.reset}  ${t.dim}vs${t.reset}  ${t.dim}${diff.right.name}${t.reset}`);
  lines.push("");
  if (!diff.changed) {
    lines.push(`  ${t.dim}No material run deltas found.${t.reset}`);
  } else {
    for (const [field, delta] of Object.entries(diff.fields)) {
      lines.push(`  ${t.bold}${field}${t.reset}  ${formatDeltaValue(delta.left)} -> ${formatDeltaValue(delta.right)}`);
    }
    if (diff.actors.added.length > 0 || diff.actors.removed.length > 0) {
      lines.push(`  ${t.bold}actors${t.reset}  +${diff.actors.added.join(", ") || "none"}  -${diff.actors.removed.join(", ") || "none"}`);
    }
    if (diff.artifactTypes.added.length > 0 || diff.artifactTypes.removed.length > 0) {
      lines.push(`  ${t.bold}artifacts${t.reset}  +${diff.artifactTypes.added.join(", ") || "none"}  -${diff.artifactTypes.removed.join(", ") || "none"}`);
    }
  }
  lines.push("");
  return lines.join("\n");
}

function formatHistoryVerification(receipt: LocalReceiptSummary): string {
  const signature = receipt.verification?.status ?? "unknown";
  const ledger = receipt.ledgerVerification?.status ?? "unknown";
  return `${signature}/${ledger}`;
}

function formatDeltaValue(value: unknown): string {
  if (value === undefined) {
    return "none";
  }
  if (typeof value === "string") {
    return value;
  }
  return JSON.stringify(value);
}
