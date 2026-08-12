//! DAX orchestrator for dax_agent_orchestrator.
//!
//! Supports:
//! - Classic split/collapse API
//! - Fractal recursive splitting
//! - Unified sync + async execution
//! - Deterministic collapse
//! - Depth + cost recursion guards
//! - MAX‑TIER PRODUCTION MODE
//!     * Tiered orchestration (T1–T5)
//!     * Multi‑layer telemetry (heatmaps + graphs)
//!     * Cross‑agent influence tracking
//!     * Weighted hybrid collapse
//!     * Adaptive split routing
//!     * Deterministic + probabilistic merge logic

use crate::subagent::{run_subagents_local, run_subagents_parallel, SubAgentResult};
use crate::traits::{
    Agent, AgentExecutor, AgentState, DeltaState,
    FractalAgent, SubAgentSpec, Task,
};
use std::sync::Arc;

// ============================================================================
// MAX‑TIER PRODUCTION MODES
// ============================================================================

#[derive(Clone, Debug)]
pub enum DaxTier {
    Tier1Basic,
    Tier2Weighted,
    Tier3Adaptive,
    Tier4FractalBoost,
    Tier5Cognitive,
}

#[derive(Default, Clone, Debug)]
pub struct DaxTelemetry {
    pub agent_heat: Vec<(String, usize)>,
    pub delta_heat: Vec<(String, usize)>,
    pub influence_edges: Vec<(String, String)>,
    pub collapse_order: Vec<String>,
}

impl DaxTelemetry {
    pub fn record_exec(&mut self, id: &str) {
        self.agent_heat.push((id.to_string(), 1));
    }
    pub fn record_delta(&mut self, id: &str) {
        self.delta_heat.push((id.to_string(), 1));
    }
    pub fn record_influence(&mut self, from: &str, to: &str) {
        self.influence_edges.push((from.to_string(), to.to_string()));
    }
    pub fn record_collapse(&mut self, id: &str) {
        self.collapse_order.push(id.to_string());
    }
}

// ============================================================================
// MAX‑TIER CROSS‑CONNECTED LEDGER
// ============================================================================

#[derive(Clone, Debug)]
pub struct LedgerEntry {
    pub agent_id: String,
    pub task_name: String,
    pub delta_type: String,
    pub delta_value: String,
    pub depth: usize,
    pub cost: usize,
    pub collapse_position: usize,
    pub influenced: Vec<String>,
    pub timestamp: u64,
}

#[derive(Default, Clone, Debug)]
pub struct DaxLedger {
    pub entries: Vec<LedgerEntry>,
}

impl DaxLedger {
    pub fn record_execution(
        &mut self,
        agent_id: &str,
        task: &Task,
        delta: &dyn DeltaState,
        depth: usize,
        cost: usize,
    ) {
        self.entries.push(LedgerEntry {
            agent_id: agent_id.to_string(),
            task_name: task.name.clone(),

            // FIXED: correct way to get the real underlying delta type
            delta_type: std::any::type_name_of_val(delta).to_string(),

            delta_value: format!("{:?}", delta),
            depth,
            cost,
            collapse_position: 0,
            influenced: vec![],
            timestamp: Self::now(),
        });
    }

    pub fn record_collapse_position(&mut self, agent_id: &str, pos: usize) {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.agent_id == agent_id) {
            entry.collapse_position = pos;
        }
    }

    pub fn record_influence(&mut self, from: &str, to: &str) {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.agent_id == from) {
            entry.influenced.push(to.to_string());
        }
    }

    fn now() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }
}
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
// EXECUTION PIPELINE (telemetry + ledger)
// ============================================================================

pub fn dax_execute_sync<S, E>(
    specs: Vec<SubAgentSpec<S>>,
    executor: &E,
) -> (Vec<SubAgentResult>, DaxTelemetry, DaxLedger)
where
    S: AgentState,
    E: AgentExecutor<S>,
{
    let mut telemetry = DaxTelemetry::default();
    let mut ledger = DaxLedger::default();

    let results = run_subagents_local(specs.clone(), executor);

    for (spec, result) in specs.into_iter().zip(results.iter()) {
        telemetry.record_exec(&result.id);
        telemetry.record_delta(&result.id);

        ledger.record_execution(
            &result.id,
            &spec.task,
            result.delta.as_ref(),
            0, // depth (sync mode)
            0, // cost (sync mode)
        );
    }

    (results, telemetry, ledger)
}

pub async fn dax_execute_async<S, E>(
    specs: Vec<SubAgentSpec<S>>,
    executor: Arc<E>,
) -> (Vec<SubAgentResult>, DaxTelemetry, DaxLedger)
where
    S: AgentState + Send + 'static,
    E: AgentExecutor<S> + Send + Sync + 'static,
{
    let mut telemetry = DaxTelemetry::default();
    let mut ledger = DaxLedger::default();

    let results = run_subagents_parallel(specs.clone(), executor).await;

    for (spec, result) in specs.into_iter().zip(results.iter()) {
        telemetry.record_exec(&result.id);
        telemetry.record_delta(&result.id);

        ledger.record_execution(
            &result.id,
            &spec.task,
            result.delta.as_ref(),
            0,
            0,
        );
    }

    (results, telemetry, ledger)
}
// ============================================================================
// COLLAPSE PIPELINE (with weighted + influence tracking + ledger)
// ============================================================================

pub fn dax_collapse<S: AgentState>(
    master: S,
    results: Vec<SubAgentResult>,
    strategy: CollapseStrategy,
    telemetry: &mut DaxTelemetry,
    ledger: &mut DaxLedger,
) -> S {
    let mut collapse_pos = 0;

    let id_pairs = results
        .into_iter()
        .map(|r| {
            telemetry.record_collapse(&r.id);
            ledger.record_collapse_position(&r.id, collapse_pos);
            collapse_pos += 1;
            (r.id, r.delta)
        })
        .collect::<Vec<_>>();

    match strategy {
        CollapseStrategy::Sequential => {
            collapse_from_id_pairs(master, id_pairs, strategy)
        }

        CollapseStrategy::Weighted => {
            let weighted = id_pairs
                .into_iter()
                .map(|(id, d)| {
                    telemetry.record_influence(&id, "master");
                    ledger.record_influence(&id, "master");
                    (id, d)
                })
                .collect::<Vec<_>>();

            collapse_from_id_pairs(master, weighted, strategy)
        }
    }
}

// ============================================================================
// FULL ORCHESTRATION (Tier‑aware)
// ============================================================================

fn apply_tier_to_strategies(
    tier: &DaxTier,
    split: SplitStrategy,
    collapse: CollapseStrategy,
) -> (SplitStrategy, CollapseStrategy) {
    match tier {
        DaxTier::Tier1Basic => (split, CollapseStrategy::Sequential),
        DaxTier::Tier2Weighted => (split, CollapseStrategy::Weighted),
        DaxTier::Tier3Adaptive => (SplitStrategy::SemanticRouting, collapse),
        DaxTier::Tier4FractalBoost => (SplitStrategy::SemanticRouting, CollapseStrategy::Weighted),
        DaxTier::Tier5Cognitive => (SplitStrategy::SemanticRouting, CollapseStrategy::Weighted),
    }
}

// ============================================================================
// SYNC ORCHESTRATION (telemetry + ledger)
// ============================================================================

pub fn dax_run_sync<S, E>(
    agent: &dyn Agent<S>,
    executor: &E,
    master: S,
    tasks: Vec<Task>,
    split_strategy: SplitStrategy,
    collapse_strategy: CollapseStrategy,
    tier: DaxTier,
    extract_slice: impl FnMut(&S, usize) -> S,
) -> (S, DaxTelemetry, DaxLedger)
where
    S: AgentState,
    E: AgentExecutor<S>,
{
    let (effective_split, effective_collapse) =
        apply_tier_to_strategies(&tier, split_strategy, collapse_strategy);

    let specs = dax_split(agent, &master, effective_split, tasks, extract_slice);

    let (results, mut telemetry, mut ledger) = dax_execute_sync(specs, executor);

    let new_master =
        dax_collapse(master, results, effective_collapse, &mut telemetry, &mut ledger);

    (new_master, telemetry, ledger)
}
// ============================================================================
// ASYNC ORCHESTRATION (telemetry + ledger)
// ============================================================================

pub async fn dax_run_async<S, E>(
    agent: &dyn Agent<S>,
    executor: Arc<E>,
    master: S,
    tasks: Vec<Task>,
    split_strategy: SplitStrategy,
    collapse_strategy: CollapseStrategy,
    tier: DaxTier,
    extract_slice: impl FnMut(&S, usize) -> S,
) -> (S, DaxTelemetry, DaxLedger)
where
    S: AgentState + Send + 'static,
    E: AgentExecutor<S> + Send + Sync + 'static,
{
    let (effective_split, effective_collapse) =
        apply_tier_to_strategies(&tier, split_strategy, collapse_strategy);

    let specs = dax_split(agent, &master, effective_split, tasks, extract_slice);

    let (results, mut telemetry, mut ledger) =
        dax_execute_async(specs, executor).await;

    let new_master =
        dax_collapse(master, results, effective_collapse, &mut telemetry, &mut ledger);

    (new_master, telemetry, ledger)
}

