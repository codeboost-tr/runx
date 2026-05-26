use std::path::Path;

use runx_parser::GraphStep;

use super::super::graph::{resolve_inputs, resolve_inputs_with_index};
use super::super::graph_index::PriorRunIndex;
use super::steps::run_step_with_inputs;
use super::{Runtime, StepRun};
use crate::RuntimeError;
use crate::adapter::SkillAdapter;
use crate::host::Host;

pub(super) fn run_step<A>(
    runtime: &Runtime<A>,
    graph_dir: &Path,
    graph_name: &str,
    step: &GraphStep,
    attempt: u32,
    prior_runs: &[StepRun],
    host: &mut dyn Host,
) -> Result<StepRun, RuntimeError>
where
    A: SkillAdapter,
{
    let inputs = resolve_inputs(step, prior_runs)?;
    run_step_with_inputs(runtime, graph_dir, graph_name, step, attempt, inputs, host)
}

pub(super) fn run_step_with_index<A>(
    runtime: &Runtime<A>,
    graph_dir: &Path,
    graph_name: &str,
    step: &GraphStep,
    attempt: u32,
    prior_run_index: &PriorRunIndex<'_>,
    host: &mut dyn Host,
) -> Result<StepRun, RuntimeError>
where
    A: SkillAdapter,
{
    let inputs = resolve_inputs_with_index(step, prior_run_index)?;
    run_step_with_inputs(runtime, graph_dir, graph_name, step, attempt, inputs, host)
}
