import type { MaturityTier } from "./store.js";

// Harness-derived inputs to the maturity decision. Mirrors
// runx-contracts::maturity::MaturitySignals.
export interface MaturitySignals {
  readonly declared_case_count: number;
  readonly all_declared_cases_passed: boolean;
  readonly has_passing_graph_case: boolean;
}

// Pure maturity decision. Mirror of runx-core::policy::compute_maturity (the
// canonical algorithm). Keep the two in sync.
//
// - "alpha" is the floor: no declared cases, or any declared case not passing.
// - "beta": every declared case passes.
// - "stable": every declared case passes and at least one passing case proves
//   the skill runs inside a graph.
export function computeMaturity(signals: MaturitySignals): MaturityTier {
  if (signals.declared_case_count === 0 || !signals.all_declared_cases_passed) {
    return "alpha";
  }
  return signals.has_passing_graph_case ? "stable" : "beta";
}
