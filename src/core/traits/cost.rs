//! Cost prediction system — MAX‑TIER
//!
//! Defines:
//! - CostPredictor (task + delta + recursion cost prediction)
//! - DefaultCostPredictor (baseline implementation)

use std::fmt::Debug;

use super::agent_state::AgentState;
use super::task::Task;
use super::delta::DeltaState;

// ============================================================================
// COST PREDICTOR TRAIT
// ============================================================================

/// Predicts the cost of executing tasks, deltas, and recursion.
pub trait CostPredictor<S: AgentState>: Send + Sync {
    /// Predict cost of a single task.
    fn predict_task_cost(&self, state: &S, task: &Task) -> usize;

    /// Predict cost of multiple tasks.
    fn predict_many(
        &self,
        state: &S,
        tasks: &[Task],
    ) -> usize {
        tasks.iter()
            .map(|t| self.predict_task_cost(state, t))
            .sum()
    }

    /// Optional: predict cost of a delta.
    fn predict_delta_cost(
        &self,
        _delta: &dyn DeltaState,
    ) -> Option<usize> {
        None
    }

    /// Optional: predict cost of recursion.
    fn predict_recursion_cost(
        &self,
        depth: usize,
        children: usize,
    ) -> Option<usize> {
        Some(depth * children)
    }
}

// ============================================================================
// DEFAULT COST PREDICTOR
// ============================================================================

#[derive(Debug)]
pub struct DefaultCostPredictor;

impl<S: AgentState> CostPredictor<S> for DefaultCostPredictor {
    fn predict_task_cost(&self, _state: &S, _task: &Task) -> usize {
        1
    }
}
