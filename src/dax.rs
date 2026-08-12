//! DAX orchestrator for dax_agent_orchestrator.
//!
//! Supports:
//! - Classic split/collapse API
//! - Fractal recursive splitting
//! - Unified sync + async execution
//! - Deterministic collapse
//! - Depth + cost recursion guards

use crate::subagent::{run_subagents_local, run_subagents_parallel, SubAgentResult};
use crate::traits::{
    Agent, AgentExecutor, AgentState, DeltaState,
    FractalAgent, SubAgentSpec, Task,
};
use std::sync::Arc;

// ============================================================================
// CLASSIC SPLIT / COLLAPSE API
// ============================================================================

#[non_exhaustive]
#[derive(Clone, Debug)]
pub enum SplitStrategy {
    RoundRobin(usize),
    SemanticRouting,
}

#[non_exhaustive]
#[derive(Clone, Debug)]
pub enum CollapseStrategy {
    Sequential,
    Weighted,
}

pub fn split<S>(
    state: &S,
    _strategy: SplitStrategy,
    tasks: Vec<Task>,
    mut extract_slice: impl FnMut(&S, usize) -> S,
) -> Vec<SubAgentSpec<S>>
where
    S: AgentState,
{
    tasks
        .into_iter()
        .enumerate()
        .map(|(i, task)| SubAgentSpec {
            id: format!("sub-{}", i),
            scoped_state: extract_slice(state, i),
            task,
        })
        .collect()
}

pub fn collapse<S, I>(mut original: S, deltas: I, _strategy: CollapseStrategy) -> S
where
    S: AgentState,
    I: IntoIterator<Item = Box<dyn DeltaState + Send>>,
{
    for delta in deltas {
        original.apply_delta(delta.as_ref());
    }
    original
}

pub fn collapse_with<S, F, I>(mut original: S, deltas: I, mut merge_fn: F) -> S
where
    S: AgentState,
    F: FnMut(&mut S, &dyn DeltaState, Option<&str>),
    I: IntoIterator<Item = (Option<String>, Box<dyn DeltaState + Send>)>,
{
    for (id_opt, delta) in deltas {
        merge_fn(&mut original, delta.as_ref(), id_opt.as_deref());
    }
    original
}

pub fn collapse_from_id_pairs<S>(
    original: S,
    id_and_deltas: Vec<(String, Box<dyn DeltaState + Send>)>,
    _strategy: CollapseStrategy,
) -> S
where
    S: AgentState,
{
    let deltas_opt = id_and_deltas
        .into_iter()
        .map(|(id, d)| (Some(id), d));

    collapse_with(original, deltas_opt, |master, delta, _id| {
        master.apply_delta(delta);
    })
}

// ============================================================================
// FRACTAL SPLIT PIPELINE
// ============================================================================

pub fn dax_split<S: AgentState>(
    agent: &dyn Agent<S>,
    state: &S,
    strategy: SplitStrategy,
    tasks: Vec<Task>,
    extract_slice: impl FnMut(&S, usize) -> S,
) -> Vec<SubAgentSpec<S>> {
    if let Some(fractal) = agent.as_fractal() {
        return dax_split_fractal(fractal.as_ref(), state.clone(), tasks);
    }
    split(state, strategy, tasks, extract_slice)
}

pub fn dax_split_fractal<S: AgentState>(
    fractal: &dyn FractalAgent<S>,
    state: S,
    tasks: Vec<Task>,
) -> Vec<SubAgentSpec<S>> {
    let mut specs = Vec::new();

    for (i, task) in tasks.into_iter().enumerate() {
        specs.push(SubAgentSpec {
            id: format!("sub-{}", i),
            scoped_state: state.clone(),
            task: task.clone(),
        });

        specs.extend(dax_expand_fractal(fractal, state.clone(), 1));
    }

    specs
}

pub fn dax_expand_fractal<S: AgentState>(
    fractal: &dyn FractalAgent<S>,
    state: S,
    depth: usize,
) -> Vec<SubAgentSpec<S>> {
    let cfg = fractal.config();

    if depth >= cfg.max_depth {
        return vec![];
    }

    if fractal.estimate_cost(&state) > cfg.max_cost {
        return vec![];
    }

    let subs = fractal.split(state.clone(), depth);

    let mut expanded = Vec::new();
    for sub in subs {
        expanded.push(sub.clone());
        expanded.extend(dax_expand_fractal(fractal, sub.scoped_state.clone(), depth + 1));
    }

    expanded
}

// ============================================================================
// EXECUTION PIPELINE
// ============================================================================

pub fn dax_execute_sync<S, E>(
    specs: Vec<SubAgentSpec<S>>,
    executor: &E,
) -> Vec<SubAgentResult>
where
    S: AgentState,
    E: AgentExecutor<S>,
{
    run_subagents_local(specs, executor)
}

pub async fn dax_execute_async<S, E>(
    specs: Vec<SubAgentSpec<S>>,
    executor: Arc<E>,
) -> Vec<SubAgentResult>
where
    S: AgentState + Send + 'static,
    E: AgentExecutor<S> + Send + Sync + 'static,
{
    run_subagents_parallel(specs, executor).await
}

// ============================================================================
// COLLAPSE PIPELINE
// ============================================================================

pub fn dax_collapse<S: AgentState>(
    master: S,
    results: Vec<SubAgentResult>,
    strategy: CollapseStrategy,
) -> S {
    let id_pairs = results
        .into_iter()
        .map(|r| (r.id, r.delta))
        .collect();

    collapse_from_id_pairs(master, id_pairs, strategy)
}

// ============================================================================
// FULL ORCHESTRATION
// ============================================================================

pub fn dax_run_sync<S, E>(
    agent: &dyn Agent<S>,
    executor: &E,
    master: S,
    tasks: Vec<Task>,
    split_strategy: SplitStrategy,
    collapse_strategy: CollapseStrategy,
    extract_slice: impl FnMut(&S, usize) -> S,
) -> S
where
    S: AgentState,
    E: AgentExecutor<S>,
{
    let specs = dax_split(agent, &master, split_strategy, tasks, extract_slice);
    let results = dax_execute_sync(specs, executor);
    dax_collapse(master, results, collapse_strategy)
}

pub async fn dax_run_async<S, E>(
    agent: &dyn Agent<S>,
    executor: Arc<E>,
    master: S,
    tasks: Vec<Task>,
    split_strategy: SplitStrategy,
    collapse_strategy: CollapseStrategy,
    extract_slice: impl FnMut(&S, usize) -> S,
) -> S
where
    S: AgentState + Send + 'static,
    E: AgentExecutor<S> + Send + Sync + 'static,
{
    let specs = dax_split(agent, &master, split_strategy, tasks, extract_slice);
    let results = dax_execute_async(specs, executor).await;
    dax_collapse(master, results, collapse_strategy)
}



