//! Subagent — MAX‑TIER execution unit
//!
//! Implements:
//! - Unified Agent<S>
//! - Micro‑agent routing
//! - Reflection gating
//! - Fractal recursion hooks
//! - Collapse + merge integration
//! - Cost prediction
//! - Scratchpad + DND safety
//!
//! This is the “worker agent” used by the DAX orchestrator.

use std::fmt::{Debug, Formatter};
use std::sync::Arc;

use crate::core::traits::{
    Agent,
    AgentState,
    Task,
    AgentExecutor,
    CollapseStrategy,
    MergeStrategy,
    CostPredictor,
    MicroAgentAcceptance,
    MicroAgentExecutor,
    MicroAgentFallback,
    FractalAgent,
    ReflectiveAgent,
    ScratchpadAgent,
    DoNotDoAgent,
    CapabilityIntrospection,
    AgentCapabilities,
    ReflectionData,
    FractalSplit,
    Scratchpad,
    DoNotDoGraph,
    micro::MicroRouteDecision,
};

use crate::core::traits::delta::DeltaState;

// ============================================================================
// SUBAGENT STRUCTURE
// ============================================================================

pub struct SubAgent<S: AgentState> {
    pub name: String,

    pub collapse: Arc<dyn CollapseStrategy<S> + Send + Sync>,
    pub merge: Arc<dyn MergeStrategy + Send + Sync>,
    pub cost: Arc<dyn CostPredictor<S> + Send + Sync>,
    pub executor: Arc<dyn AgentExecutor<S> + Send + Sync>,

    pub scratchpad: Scratchpad,
    pub dnd: DoNotDoGraph,
}

// Manual Debug implementation (fixes E0277)
impl<S: AgentState> Debug for SubAgent<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SubAgent")
            .field("name", &self.name)
            .field("collapse", &"<dyn CollapseStrategy>")
            .field("merge", &"<dyn MergeStrategy>")
            .field("cost", &"<dyn CostPredictor>")
            .field("executor", &"<dyn AgentExecutor>")
            .field("scratchpad", &self.scratchpad)
            .field("dnd", &self.dnd)
            .finish()
    }
}

impl<S: AgentState> SubAgent<S> {
    pub fn new(
        name: impl Into<String>,
        collapse: Arc<dyn CollapseStrategy<S> + Send + Sync>,
        merge: Arc<dyn MergeStrategy + Send + Sync>,
        cost: Arc<dyn CostPredictor<S> + Send + Sync>,
        executor: Arc<dyn AgentExecutor<S> + Send + Sync>,
    ) -> Self {
        Self {
            name: name.into(),
            collapse,
            merge,
            cost,
            executor,
            scratchpad: Scratchpad::new(),
            dnd: DoNotDoGraph::new(),
        }
    }
}

// ============================================================================
// MICRO‑AGENT ACCEPTANCE
// ============================================================================

impl<S: AgentState> MicroAgentAcceptance<S> for SubAgent<S> {
    fn should_accept(&self, _state: &S, task: &Task) -> MicroRouteDecision {
        if let Some(reason) = self.dnd.is_forbidden(task) {
            return MicroRouteDecision::reject(Some(reason));
        }

        MicroRouteDecision::accept(None)
    }

    fn priority(&self) -> f32 {
        1.0
    }

    fn name(&self) -> Option<String> {
        Some(self.name.clone())
    }
}

// ============================================================================
// MICRO‑AGENT EXECUTION
// ============================================================================

impl<S: AgentState> MicroAgentExecutor<S> for SubAgent<S> {
    fn execute(&self, state: &S, task: &Task) -> Box<dyn DeltaState + Send> {
        // AgentExecutor::run is expected to take owned state/task or clones;
        // we clone here to match the common executor signature used elsewhere.
        self.executor.run(state.clone(), task.clone())
    }
}

// ============================================================================
// MICRO‑AGENT FALLBACK
// ============================================================================

impl<S: AgentState> MicroAgentFallback<S> for SubAgent<S> {
    fn fallback(&self, _state: &S, _task: &Task) -> Option<Box<dyn DeltaState + Send>> {
        None
    }

    fn reason(&self) -> Option<String> {
        Some("fallback not implemented".to_string())
    }
}

// ============================================================================
// FRACTAL AGENT
// ============================================================================

impl<S: AgentState> FractalAgent<S> for SubAgent<S> {
    fn split_task(&self, _state: &S, task: &Task, _depth: usize) -> Option<FractalSplit> {
        Some(FractalSplit::new(vec![task.clone()]))
    }
}

// ============================================================================
// REFLECTIVE AGENT
// ============================================================================

impl<S: AgentState> ReflectiveAgent<S> for SubAgent<S> {
    fn reflect(&self, _state: &S, task: &Task) -> ReflectionData {
        let mut r = ReflectionData::new();
        r.assumptions.push(format!("Task '{}' is safe", task.name));
        r
    }
}

// ============================================================================
// SCRATCHPAD AGENT
// ============================================================================

impl<S: AgentState> ScratchpadAgent<S> for SubAgent<S> {
    fn scratchpad(&self) -> &Scratchpad {
        &self.scratchpad
    }

    fn scratchpad_mut(&mut self) -> &mut Scratchpad {
        &mut self.scratchpad
    }
}

// ============================================================================
// DND AGENT
// ============================================================================

impl<S: AgentState> DoNotDoAgent<S> for SubAgent<S> {
    fn dnd_graph(&self) -> &DoNotDoGraph {
        &self.dnd
    }

    fn dnd_graph_mut(&mut self) -> &mut DoNotDoGraph {
        &mut self.dnd
    }
}

// ============================================================================
// CAPABILITY INTROSPECTION
// ============================================================================

impl<S: AgentState> CapabilityIntrospection<S> for SubAgent<S> {
    fn capabilities(&self) -> AgentCapabilities {
        let mut c = AgentCapabilities::new();
        c.can_reflect = true;
        c.can_fractal = true;
        c.has_scratchpad = true;
        c.has_dnd = true;
        c.can_merge = true;
        c.can_collapse = true;
        c.can_predict_cost = true;
        c
    }
}

// ============================================================================
// UNIFIED AGENT IMPLEMENTATION
// ============================================================================

impl<S: AgentState> Agent<S> for SubAgent<S> {
    fn name(&self) -> &str {
        &self.name
    }

    fn collapse_strategy(&self) -> Arc<dyn CollapseStrategy<S> + Send + Sync> {
        self.collapse.clone()
    }

    fn merge_strategy(&self) -> Arc<dyn MergeStrategy + Send + Sync> {
        self.merge.clone()
    }

    fn cost_predictor(&self) -> Arc<dyn CostPredictor<S> + Send + Sync> {
        self.cost.clone()
    }

    fn executor(&self) -> Arc<dyn AgentExecutor<S> + Send + Sync> {
        self.executor.clone()
    }
}
