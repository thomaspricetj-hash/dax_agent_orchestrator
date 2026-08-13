//! Do‑Not‑Do (DND) safety graph — MAX‑TIER
//!
//! Defines:
//! - ForbiddenAction (task + reason)
//! - DoNotDoGraph (collection of forbidden actions)
//! - DoNotDoAgent (agents with safety gating)

use std::fmt::Debug;

use super::agent_state::AgentState;
use super::task::Task;

// ============================================================================
// FORBIDDEN ACTION
// ============================================================================

#[derive(Clone, Debug)]
pub struct ForbiddenAction {
    pub task_name: String,
    pub reason: String,
}

impl ForbiddenAction {
    pub fn new(task: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            task_name: task.into(),
            reason: reason.into(),
        }
    }
}

// ============================================================================
// DO‑NOT‑DO GRAPH
// ============================================================================

#[derive(Clone, Debug)]
pub struct DoNotDoGraph {
    pub forbidden: Vec<ForbiddenAction>,
}

impl DoNotDoGraph {
    pub fn new() -> Self {
        Self { forbidden: Vec::new() }
    }

    pub fn forbid(&mut self, task: impl Into<String>, reason: impl Into<String>) {
        self.forbidden.push(ForbiddenAction::new(task, reason));
    }

    pub fn is_forbidden(&self, task: &Task) -> Option<String> {
        let name = task.name.clone();
        self.forbidden
            .iter()
            .find(|f| f.task_name == name)
            .map(|f| f.reason.clone())
    }
}

// ============================================================================
// DO‑NOT‑DO AGENT TRAIT
// ============================================================================

pub trait DoNotDoAgent<S: AgentState>: Send + Sync {
    fn dnd_graph(&self) -> &DoNotDoGraph;

    fn dnd_graph_mut(&mut self) -> &mut DoNotDoGraph;

    /// Should this agent run this task?
    fn allowed(&self, task: &Task) -> bool {
        self.dnd_graph().is_forbidden(task).is_none()
    }
}
