//! Reflection system — MAX‑TIER
//!
//! Defines:
//! - ReflectionData (assumptions, risks, predicted delta, gating)
//! - ReflectiveAgent (pre‑execution reflection trait)

use std::fmt::Debug;

use super::agent_state::AgentState;
use super::task::Task;

// ============================================================================
// REFLECTION DATA
// ============================================================================

#[derive(Clone, Debug)]
pub struct ReflectionData {
    pub assumptions: Vec<String>,
    pub risks: Vec<String>,
    pub predicted_delta: Option<String>,
    pub should_run: bool,
    pub reason: Option<String>,
}

impl ReflectionData {
    pub fn new() -> Self {
        Self {
            assumptions: Vec::new(),
            risks: Vec::new(),
            predicted_delta: None,
            should_run: true,
            reason: None,
        }
    }

    pub fn add_assumption(mut self, a: impl Into<String>) -> Self {
        self.assumptions.push(a.into());
        self
    }

    pub fn add_risk(mut self, r: impl Into<String>) -> Self {
        self.risks.push(r.into());
        self
    }

    pub fn predict(mut self, p: impl Into<String>) -> Self {
        self.predicted_delta = Some(p.into());
        self
    }

    pub fn gate(mut self, should: bool, reason: impl Into<String>) -> Self {
        self.should_run = should;
        self.reason = Some(reason.into());
        self
    }
}

// ============================================================================
// REFLECTIVE AGENT TRAIT
// ============================================================================

/// Reflection trait.
/// Agents can reflect before executing.
pub trait ReflectiveAgent<S: AgentState>: Send + Sync {
    /// Whether this agent supports reflection.
    fn can_reflect(&self) -> bool {
        true
    }

    /// Produce reflection data.
    fn reflect(
        &self,
        state: &S,
        task: &Task,
    ) -> ReflectionData;

    /// Optional: reflection depth limit.
    fn max_reflection_depth(&self) -> usize {
        8
    }
}
