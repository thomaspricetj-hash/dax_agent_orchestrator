//! Fractal recursion system — MAX‑TIER
//!
//! Defines:
//! - FractalSplit (sub‑task grouping)
//! - FractalAgent (recursive agent trait)

use std::fmt::Debug;

use super::agent_state::AgentState;
use super::task::Task;

// ============================================================================
// FRACTAL SPLIT
// ============================================================================

/// Describes how an agent splits a task into sub‑tasks.
#[derive(Clone, Debug)]
pub struct FractalSplit {
    pub sub_tasks: Vec<Task>,
    pub reason: Option<String>,
    pub depth_increase: usize,
}

impl FractalSplit {
    pub fn new(sub_tasks: Vec<Task>) -> Self {
        Self {
            sub_tasks,
            reason: None,
            depth_increase: 1,
        }
    }
}

// ============================================================================
// FRACTAL AGENT TRAIT
// ============================================================================

/// Fractal recursion trait.
/// Agents that can recursively split tasks.
pub trait FractalAgent<S: AgentState>: Send + Sync {
    /// Whether this agent supports fractal recursion.
    fn can_fractal(&self) -> bool {
        true
    }

    /// Split a task into sub‑tasks.
    fn split_task(
        &self,
        state: &S,
        task: &Task,
        depth: usize,
    ) -> Option<FractalSplit>;

    /// Optional: limit recursion depth.
    fn max_fractal_depth(&self) -> usize {
        32
    }

    /// Optional: limit recursion cost.
    fn max_fractal_cost(&self) -> usize {
        10_000
    }
}
