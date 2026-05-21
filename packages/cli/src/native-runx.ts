import { spawn } from "node:child_process";
import { existsSync } from "node:fs";
import path from "node:path";
import process from "node:process";

import { errorMessage } from "@runxhq/core/util";

export interface NativeRunxProcessResult {
  readonly status: number | null;
  readonly stdout: string;
  readonly stderr: string;
}

export interface NativeRunxOptions {
  readonly env: NodeJS.ProcessEnv;
  readonly cwd?: string;
  readonly timeoutMs?: number;
}

export async function runNativeRunxJson(
  args: readonly string[],
  options: NativeRunxOptions,
): Promise<unknown> {
  const result = await runNativeRunx(args, options);
  if (result.status !== 0) {
    throw nativeRunxError(args, result);
  }
  try {
    return JSON.parse(result.stdout);
  } catch (error) {
    throw new Error(`native runx ${args.join(" ")} returned invalid JSON: ${errorMessage(error)}`);
  }
}

export async function runNativeRunx(
  args: readonly string[],
  options: NativeRunxOptions,
): Promise<NativeRunxProcessResult> {
  const timeoutMs = options.timeoutMs ?? parsePositiveInt(options.env.RUNX_RUST_CLI_TIMEOUT_MS);
  return await spawnNativeRunx({
    command: resolveNativeRunxBinary(options.env),
    args,
    cwd: options.cwd ?? options.env.RUNX_CWD ?? process.cwd(),
    env: {
      ...process.env,
      ...options.env,
      NO_COLOR: "1",
      RUNX_RUST_CLI: "1",
    },
    timeoutMs,
  });
}

function resolveNativeRunxBinary(env: NodeJS.ProcessEnv): string {
  for (const candidate of [
    env.RUNX_RUST_CLI_BIN,
    env.RUNX_RUST_REGISTRY_BIN,
    path.join(process.cwd(), "crates", "target", "debug", "runx"),
    path.join(process.cwd(), "crates", "target", "release", "runx"),
    path.join(process.cwd(), "oss", "crates", "target", "debug", "runx"),
    path.join(process.cwd(), "oss", "crates", "target", "release", "runx"),
  ]) {
    if (candidate && (candidate === "runx" || existsSync(candidate))) {
      return candidate;
    }
  }
  return "runx";
}

interface SpawnNativeRunxOptions {
  readonly command: string;
  readonly args: readonly string[];
  readonly cwd: string;
  readonly env: NodeJS.ProcessEnv;
  readonly timeoutMs?: number;
}

function spawnNativeRunx(options: SpawnNativeRunxOptions): Promise<NativeRunxProcessResult> {
  return new Promise((resolve, reject) => {
    const child = spawn(options.command, options.args, {
      cwd: options.cwd,
      env: options.env,
      stdio: ["ignore", "pipe", "pipe"],
    });
    let settled = false;
    let timedOut = false;
    let stdout = "";
    let stderr = "";
    let killTimer: NodeJS.Timeout | undefined;

    const timer = options.timeoutMs === undefined
      ? undefined
      : setTimeout(() => {
          if (settled) return;
          timedOut = true;
          child.kill("SIGTERM");
          killTimer = setTimeout(() => {
            if (settled) return;
            settled = true;
            child.kill("SIGKILL");
            reject(new Error(`native runx ${options.args.join(" ")} timed out after ${options.timeoutMs}ms.`));
          }, 1_000);
        }, options.timeoutMs);

    const clearTimers = () => {
      if (timer) clearTimeout(timer);
      if (killTimer) clearTimeout(killTimer);
    };

    child.stdout.setEncoding("utf8");
    child.stderr.setEncoding("utf8");
    child.stdout.on("data", (chunk: string) => {
      stdout += chunk;
    });
    child.stderr.on("data", (chunk: string) => {
      stderr += chunk;
    });
    child.on("error", (error) => {
      if (settled) return;
      settled = true;
      clearTimers();
      reject(new Error(`failed to spawn native runx '${options.command}': ${error.message}`));
    });
    child.on("close", (status) => {
      if (settled) return;
      settled = true;
      clearTimers();
      if (timedOut) {
        reject(new Error(`native runx ${options.args.join(" ")} timed out after ${options.timeoutMs ?? "unknown"}ms.`));
        return;
      }
      resolve({ status, stdout, stderr });
    });
  });
}

function nativeRunxError(args: readonly string[], result: NativeRunxProcessResult): Error {
  return new Error(
    `native runx ${args.join(" ")} failed with exit ${result.status}: ${firstNonEmpty(result.stderr, result.stdout, "no output")}`,
  );
}

function parsePositiveInt(value: string | undefined): number | undefined {
  if (!value) return undefined;
  const parsed = Number.parseInt(value, 10);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : undefined;
}

function firstNonEmpty(...values: readonly string[]): string {
  for (const value of values) {
    const trimmed = value.trim();
    if (trimmed) return trimmed;
  }
  return "";
}
