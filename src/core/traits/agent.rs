//! Unified Agent Trait — MAX‑TIER
//!
//! This is the top‑level agent interface that ties together:
//! - Micro‑agents (acceptance + fallback)
//! - Fractal recursion
//! - Reflection
//! - Collapse strategies
//! - Merge strategies
//! - Cost prediction
//! - Scratchpad
//! - DND safety graph
//! - Capability introspection
//! - Executors
//!
//! Every agent in the system implements this trait.

use std::fmt::Debug;
use std::sync::Arc;

use super::agent_state::AgentState;
use super::task::Task;
use super::delta::DeltaState;

use super::micro::{MicroAgentAcceptance, MicroAgentFallback};
use super::fractal::FractalAgent;
use super::reflection::ReflectiveAgent;
use super::collapse::CollapseStrategy;
use crate::core::traits::MergeStrategy;


use super::cost::CostPredictor;
use super::scratchpad::ScratchpadAgent;
use super::dnd::DoNotDoAgent;
use super::capabilities::CapabilityIntrospection;
use super::executors::AgentExecutor;

// ============================================================================
// UNIFIED AGENT TRAIT
// ============================================================================
//
// NOTE: We intentionally do NOT require `MicroAgentExecutor` here. That trait
// represents a separate micro‑executor capability whose `execute` method has a
// different signature and would conflict with the top‑level `Agent::execute`.
// The orchestrator should call `Agent::execute` for end‑to‑end runs and may
// call `MicroAgentExecutor::execute` explicitly when invoking micro agents.

pub trait Agent<S>:
    MicroAgentAcceptance<S>
    + MicroAgentFallback<S>
    + FractalAgent<S>
    + ReflectiveAgent<S>
    + ScratchpadAgent<S>
    + DoNotDoAgent<S>
    + CapabilityIntrospection<S>
    + Send
    + Sync
    + Debug
where
    S: AgentState,
{
    /// The agent's name.
    fn name(&self) -> &str;

    /// Collapse strategy (deterministic, weighted, multi‑stage).
    fn collapse_strategy(&self) -> Arc<dyn CollapseStrategy<S> + Send + Sync>;

    /// Merge strategy (deterministic, weighted, multi‑agent).
    fn merge_strategy(&self) -> Arc<dyn MergeStrategy + Send + Sync>;

    /// Cost predictor (task + delta + recursion).
    fn cost_predictor(&self) -> Arc<dyn CostPredictor<S> + Send + Sync>;

    /// Executor (local, parallel, or custom).
    fn executor(&self) -> Arc<dyn AgentExecutor<S> + Send + Sync>;

    /// Execute a task end‑to‑end.
    ///
    /// Default implementation delegates to the configured executor. Keeping
    /// this method on `Agent` avoids ambiguity with `MicroAgentExecutor::execute`.
    fn execute(&self, state: S, task: Task) -> Box<dyn DeltaState + Send> {
        self.executor().run(state, task)
    }
}
