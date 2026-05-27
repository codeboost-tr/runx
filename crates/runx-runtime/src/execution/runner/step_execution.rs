use std::path::Path;

use runx_parser::GraphStep;

use super::super::graph::{LoadedStepSkill, resolve_inputs, resolve_inputs_with_index};
use super::super::graph_index::PriorRunIndex;
use super::steps::run_step_with_inputs;
use super::{Runtime, StepRun};
use crate::RuntimeError;
use crate::adapter::SkillAdapter;
use crate::host::Host;

pub(super) fn run_step_with_loaded_skill<A>(
    runtime: &Runtime<A>,
    graph_dir: &Path,
    graph_name: &str,
    step: &GraphStep,
    attempt: u32,
    loaded_skill: Option<LoadedStepSkill>,
    prior_runs: &[StepRun],
    host: &mut dyn Host,
) -> Result<StepRun, RuntimeError>
where
    A: SkillAdapter,
{
    let inputs = resolve_inputs(step, prior_runs)?;
    run_step_with_loaded_skill_inputs(
        runtime,
        graph_dir,
        graph_name,
        step,
        attempt,
        loaded_skill,
        inputs,
        host,
    )
}

pub(super) fn run_step_with_loaded_skill_index<A>(
    runtime: &Runtime<A>,
    graph_dir: &Path,
    graph_name: &str,
    step: &GraphStep,
    attempt: u32,
    loaded_skill: Option<LoadedStepSkill>,
    prior_run_index: &PriorRunIndex<'_>,
    host: &mut dyn Host,
) -> Result<StepRun, RuntimeError>
where
    A: SkillAdapter,
{
    let inputs = resolve_inputs_with_index(step, prior_run_index)?;
    run_step_with_loaded_skill_inputs(
        runtime,
        graph_dir,
        graph_name,
        step,
        attempt,
        loaded_skill,
        inputs,
        host,
    )
}

fn run_step_with_loaded_skill_inputs<A>(
    runtime: &Runtime<A>,
    graph_dir: &Path,
    graph_name: &str,
    step: &GraphStep,
    attempt: u32,
    loaded_skill: Option<LoadedStepSkill>,
    inputs: runx_contracts::JsonObject,
    host: &mut dyn Host,
) -> Result<StepRun, RuntimeError>
where
    A: SkillAdapter,
{
    if let Some(skill) = loaded_skill {
        return super::steps::run_step_with_loaded_skill_inputs(
            runtime, graph_dir, graph_name, step, attempt, skill, inputs, host,
        );
    }
    run_step_with_inputs(runtime, graph_dir, graph_name, step, attempt, inputs, host)
}
