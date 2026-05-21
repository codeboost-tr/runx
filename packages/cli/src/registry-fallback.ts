import {
  resolveRunxGlobalHomeDir,
  resolveRunxRegistryPath,
  resolveRunxRegistryTarget,
} from "@runxhq/core/config";
import type { SkillSearchResult } from "@runxhq/core/marketplaces";
import {
  createHttpCachedRegistryStore,
  createFileRegistryStore,
  searchRemoteRegistry,
  searchRegistry,
  type RegistryStore as CoreRegistryStore,
} from "@runxhq/core/registry";

import type { RegistryStore as CliRegistryStore } from "./cli-runtime-contracts.js";
import { ensureRunxInstallState } from "./runx-state.js";

export async function searchRegistryFallback(
  query: string,
  env: NodeJS.ProcessEnv,
  registryOverride?: string,
): Promise<readonly SkillSearchResult[]> {
  const registryTarget = resolveRunxRegistryTarget(env, { registry: registryOverride });
  if (registryTarget.mode === "remote") {
    return await searchRemoteRegistry(query, { baseUrl: registryTarget.registryUrl });
  }
  return await searchRegistry(createFileRegistryStore(registryTarget.registryPath), query, {
    registryUrl: registryTarget.registryUrl,
  });
}

export async function resolveCliRegistryStoreForGraphs(env: NodeJS.ProcessEnv): Promise<CliRegistryStore | undefined> {
  const target = resolveRunxRegistryTarget(env);
  if (target.mode === "local") {
    return toCliRegistryStore(createFileRegistryStore(target.registryPath));
  }
  if (!target.registryUrl) {
    return undefined;
  }
  const globalHomeDir = resolveRunxGlobalHomeDir(env);
  const install = await ensureRunxInstallState(globalHomeDir);
  return toCliRegistryStore(
    createHttpCachedRegistryStore({
      remoteBaseUrl: target.registryUrl,
      cache: createFileRegistryStore(resolveRunxRegistryPath(env)),
      installationId: install.state.installation_id,
      channel: "cli-graph",
    }),
  );
}

function toCliRegistryStore(store: CoreRegistryStore): CliRegistryStore {
  return {
    getVersion: async (skillId, version) => await store.getVersion(skillId, version),
    listVersions: async (skillId) => await store.listVersions(skillId),
    listSkills: async () => await store.listSkills(),
  };
}
