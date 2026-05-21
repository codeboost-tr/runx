export const marketplacesPackage = "@runxhq/core/marketplaces";

export type { SkillRunnerMode, SkillSearchResult, SkillSearchSource, SkillSearchTrustTier } from "../registry/search-result.js";
import type { SkillSearchResult } from "../registry/search-result.js";

export interface MarketplaceSearchOptions {
  readonly limit?: number;
}

export interface MarketplaceAdapter {
  readonly source: string;
  readonly label: string;
  readonly search: (query: string, options?: MarketplaceSearchOptions) => Promise<readonly SkillSearchResult[]>;
  readonly resolve?: (ref: string, options?: { readonly version?: string }) => Promise<{
    readonly markdown: string;
    readonly profileDocument?: string;
    readonly result: SkillSearchResult;
  } | undefined>;
}

export async function searchMarketplaceAdapters(
  adapters: readonly MarketplaceAdapter[],
  query: string,
  options: MarketplaceSearchOptions = {},
): Promise<readonly SkillSearchResult[]> {
  const results = await Promise.all(adapters.map((adapter) => adapter.search(query, options)));
  return results.flat().slice(0, options.limit ?? 20);
}

export { createFixtureMarketplaceAdapter } from "./fixture.js";
export {
  isMarketplaceRef,
  parseMarketplaceRef,
  resolveMarketplaceSkill,
  type MarketplaceResolvedSkill,
  type MarketplaceResolveOptions,
  type MarketplaceResolver,
} from "./resolve.js";
