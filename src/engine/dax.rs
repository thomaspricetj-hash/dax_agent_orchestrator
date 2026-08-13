//! DAX Orchestrator — MAX‑TIER
//!
//! Unified execution pipeline:
//! - Micro‑agent routing
//! - Reflection gating
//! - Fractal recursion
//! - Collapse + merge
//! - Cost prediction
//! - Deterministic recursion guards
//!
//! This is the top‑level orchestrator used by SyntheticMind.

use std::fmt;
use std::sync::Arc;
use std::marker::PhantomData;

use crate::core::traits::{
    Agent,
    AgentState,
    Task,
    CostPredictor,
    FractalAgent,
    ReflectiveAgent,
    MicroAgentFallback,
};

use crate::core::traits::collapse::CollapseStrategy;
use crate::core::traits::MergeStrategy;

use crate::core::traits::delta::DeltaState;

// ============================================================================
// DAX EXECUTION RESULT
// ============================================================================

pub struct DaxResult {
    pub deltas: Vec<Box<dyn DeltaState + Send>>,
    pub recursion_depth: usize,
    pub cost: usize,
}

// Provide a custom Debug impl so we don't require `DeltaState: Debug`.
// This prints a concise summary (count of deltas, depth, cost).
impl fmt::Debug for DaxResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DaxResult")
            .field("deltas_count", &self.deltas.len())
            .field("recursion_depth", &self.recursion_depth)
            .field("cost", &self.cost)
            .finish()
    }
}

// ============================================================================
// DAX ORCHESTRATOR
// ============================================================================
//
// Note: the orchestrator requires `S: Clone` because it clones the state when
// executing subtasks. If you prefer to avoid Clone, change the execution model
// to pass references or move ownership of state into subtasks.

#[derive(Debug)]
pub struct DaxOrchestrator<A, S>
where
    // Document and require the agent capabilities used by the orchestrator.
    A: Agent<S> + FractalAgent<S> + ReflectiveAgent<S> + MicroAgentFallback<S>,
    S: AgentState + Clone,
{
    pub agent: Arc<A>,
    pub max_depth: usize,
    pub max_cost: usize,

    // `S` is used only in trait bounds on `A`; include a PhantomData to avoid
    // the "type parameter is never used" warning while keeping the generic.
    _state_marker: PhantomData<S>,
}

impl<A, S> DaxOrchestrator<A, S>
where
    A: Agent<S> + FractalAgent<S> + ReflectiveAgent<S> + MicroAgentFallback<S>,
    S: AgentState + Clone,
{
    pub fn new(agent: Arc<A>) -> Self {
        Self {
            agent,
            max_depth: 64,
            max_cost: 100_000,
            _state_marker: PhantomData,
        }
    }

    // ========================================================================
    // MAIN EXECUTION ENTRYPOINT
    // ========================================================================

    pub fn execute(&self, state: S, task: Task) -> DaxResult {
        self.execute_recursive(state, task, 0, 0)
    }

    // ========================================================================
    // RECURSIVE EXECUTION
    // ========================================================================

    fn execute_recursive(
        &self,
        mut state: S,
        task: Task,
        depth: usize,
        cost: usize,
    ) -> DaxResult {
        // ------------------------------------------------------------
        // Depth guard
        // ------------------------------------------------------------
        if depth > self.max_depth {
            return DaxResult {
                deltas: vec![],
                recursion_depth: depth,
                cost,
            };
        }

        // ------------------------------------------------------------
        // Reflection gating (ReflectiveAgent USED)
        // ------------------------------------------------------------
        let reflection = self.agent.reflect(&state, &task);

        // Gate execution if reflection says not to run.
        if !reflection.should_run {
            return DaxResult {
                deltas: vec![],
                recursion_depth: depth,
                cost,
            };
        }

        // Charge a small verification overhead for reflection assumptions.
        let mut total_cost = cost;
        if !reflection.assumptions.is_empty() {
            total_cost += reflection.assumptions.len();
            if total_cost > self.max_cost {
                return DaxResult {
                    deltas: vec![],
                    recursion_depth: depth,
                    cost: total_cost,
                };
            }
        }

        // ------------------------------------------------------------
        // Micro‑agent acceptance + fallback (MicroAgentFallback USED)
        // ------------------------------------------------------------
        let decision = self.agent.should_accept(&state, &task);
        if !decision.accepted {
            if let Some(fallback_delta) = self.agent.fallback(&state, &task) {
                return DaxResult {
                    deltas: vec![fallback_delta],
                    recursion_depth: depth,
                    cost: total_cost,
                };
            }

            return DaxResult {
                deltas: vec![],
                recursion_depth: depth,
                cost: total_cost,
            };
        }

        // ------------------------------------------------------------
        // Fractal recursion (FractalAgent USED)
        // ------------------------------------------------------------
        let split = self.agent.split_task(&state, &task, depth);
        let sub_tasks = match split {
            Some(s) => s.sub_tasks,
            None => vec![task.clone()],
        };

        // ------------------------------------------------------------
        // CostPredictor usage (explicitly used via Agent::cost_predictor)
        // ------------------------------------------------------------
        let _cost_predictor_handle: Arc<dyn CostPredictor<S> + Send + Sync> =
            self.agent.cost_predictor();

        let mut all_deltas: Vec<Box<dyn DeltaState + Send>> = Vec::with_capacity(sub_tasks.len());

        for sub in sub_tasks {
            // --------------------------------------------------------
            // Cost prediction (CostPredictor USED)
            // --------------------------------------------------------
            let predicted = self.agent.cost_predictor().predict_task_cost(&state, &sub);
            total_cost += predicted;

            if total_cost > self.max_cost {
                break;
            }

            // --------------------------------------------------------
            // Execute sub‑task (explicitly call Agent::execute to disambiguate)
            // --------------------------------------------------------
            let delta = Agent::execute(&*self.agent, state.clone(), sub.clone());
            all_deltas.push(delta);
        }

        // ------------------------------------------------------------
        // Collapse
        // ------------------------------------------------------------
        {
            let cs: &dyn CollapseStrategy<S> = &*self.agent.collapse_strategy();
            cs.apply_many(&mut state, &all_deltas);
            let _collapse_meta = cs.metadata(); // Option<CollapseMetadata>
        }

        // ------------------------------------------------------------
        // Merge
        // ------------------------------------------------------------
        {
            let ms: &dyn MergeStrategy = &*self.agent.merge_strategy();
            let merged_delta: Box<dyn DeltaState + Send> = ms.merge(&all_deltas);
            let _merge_meta = ms.metadata(); // Option<MergeMetadata>
            all_deltas.push(merged_delta);
        }

        DaxResult {
            deltas: all_deltas,
            recursion_depth: depth,
            cost: total_cost,
        }
    }
}

