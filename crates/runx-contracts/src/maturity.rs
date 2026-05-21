//! Skill maturity tier and the signals it is computed from.
//!
//! Maturity describes how ready a *skill* is (tests, graph integration). It is
//! orthogonal to trust, which describes the *author*. The tier is computed from
//! harness signals at event points (publish, harness seal) and stored on the
//! registry record; readers never recompute it.

use serde::{Deserialize, Serialize};

/// How ready a skill is. `Alpha` is the floor for any published skill.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaturityTier {
    /// Published; harness incomplete or not all declared cases pass.
    #[default]
    Alpha,
    /// Every declared harness case passes.
    Beta,
    /// Every declared case passes and at least one passing case proves the
    /// skill runs inside a graph.
    Stable,
}

/// The harness-derived inputs to the maturity decision. Callers extract these
/// at an event point; the decision itself stays pure.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaturitySignals {
    /// Number of harness cases the skill declares.
    pub declared_case_count: usize,
    /// Every declared case ran and passed.
    pub all_declared_cases_passed: bool,
    /// At least one passing case targets a graph that includes this skill.
    pub has_passing_graph_case: bool,
}
