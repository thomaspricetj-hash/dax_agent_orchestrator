//! Collapse system — MAX‑TIER
//!
//! Defines:
//! - CollapseMetadata
//! - CollapseStage
//! - CollapseStrategy
//! - DeterministicCollapse
//! - WeightedCollapse
//! - MultiStageCollapse
//! - CollapseAdapter

use std::fmt::Debug;

use super::agent_state::AgentState;
use super::delta::DeltaState;

// ============================================================================
// COLLAPSE METADATA
// ============================================================================

#[derive(Clone, Debug)]
pub struct CollapseMetadata {
    pub strategy: String,
    pub stages: usize,
    pub total_weight: f32,
    pub provenance: Option<String>,
}

impl CollapseMetadata {
    pub fn new(strategy: &str) -> Self {
        Self {
            strategy: strategy.to_string(),
            stages: 1,
            total_weight: 1.0,
            provenance: None,
        }
    }
}

// ============================================================================
// COLLAPSE STAGE
// ============================================================================

#[derive(Clone, Debug)]
pub struct CollapseStage {
    pub name: String,
    pub description: Option<String>,
    pub weight_multiplier: f32,
}

impl CollapseStage {
    pub fn new(name: &str, weight_multiplier: f32) -> Self {
        Self {
            name: name.to_string(),
            description: None,
            weight_multiplier,
        }
    }
}

// ============================================================================
// COLLAPSE STRATEGY TRAIT
// ============================================================================

pub trait CollapseStrategy<S: AgentState>: Send + Sync {
    fn apply_single(&self, state: &mut S, delta: &dyn DeltaState);

    fn apply_many(&self, state: &mut S, deltas: &[Box<dyn DeltaState + Send>]) {
        for d in deltas {
            self.apply_single(state, d.as_ref());
        }
    }

    fn metadata(&self) -> Option<CollapseMetadata> {
        None
    }

    fn stages(&self) -> Option<Vec<CollapseStage>> {
        None
    }

    fn predict(&self, _deltas: &[Box<dyn DeltaState + Send>]) -> Option<String> {
        None
    }
}

// ============================================================================
// DETERMINISTIC COLLAPSE
// ============================================================================

#[derive(Debug)]
pub struct DeterministicCollapse;

impl<S: AgentState> CollapseStrategy<S> for DeterministicCollapse {
    fn apply_single(&self, state: &mut S, delta: &dyn DeltaState) {
        state.apply_delta(delta);
    }

    fn metadata(&self) -> Option<CollapseMetadata> {
        Some(CollapseMetadata::new("deterministic"))
    }
}

// ============================================================================
// WEIGHTED COLLAPSE
// ============================================================================

#[derive(Debug)]
pub struct WeightedCollapse;

impl<S: AgentState> CollapseStrategy<S> for WeightedCollapse {
    fn apply_single(&self, state: &mut S, delta: &dyn DeltaState) {
        // Apply the delta normally.
        state.apply_delta(delta);

        // Read weight via the safe DeltaState accessor (no trait-object downcast).
        let w = delta.weight().unwrap_or(1.0);
        let _w = w; // placeholder for weighted logic
    }

    fn metadata(&self) -> Option<CollapseMetadata> {
        Some(CollapseMetadata::new("weighted"))
    }
}

// ============================================================================
// MULTI‑STAGE COLLAPSE
// ============================================================================

#[derive(Debug)]
pub struct MultiStageCollapse {
    pub stages: Vec<CollapseStage>,
}

impl MultiStageCollapse {
    pub fn new(stages: Vec<CollapseStage>) -> Self {
        Self { stages }
    }
}

impl<S: AgentState> CollapseStrategy<S> for MultiStageCollapse {
    fn apply_single(&self, state: &mut S, delta: &dyn DeltaState) {
        for stage in &self.stages {
            state.apply_delta(delta);

            let _mult = stage.weight_multiplier; // placeholder
        }
    }

    fn metadata(&self) -> Option<CollapseMetadata> {
        let mut meta = CollapseMetadata::new("multi_stage");
        meta.stages = self.stages.len();
        Some(meta)
    }

    fn stages(&self) -> Option<Vec<CollapseStage>> {
        Some(self.stages.clone())
    }
}

// ============================================================================
// COLLAPSE ADAPTER
// ============================================================================

pub trait CollapseAdapter<S: AgentState>: Send + Sync {
    fn collapse(&self, state: &mut S, deltas: &[Box<dyn DeltaState + Send>]) -> CollapseMetadata;
}

// Explicit impls for the concrete strategies to avoid coherence conflicts.
// Return concrete metadata directly to avoid ambiguous trait-method resolution.

impl<S: AgentState> CollapseAdapter<S> for DeterministicCollapse {
    fn collapse(&self, state: &mut S, deltas: &[Box<dyn DeltaState + Send>]) -> CollapseMetadata {
        self.apply_many(state, deltas);
        CollapseMetadata::new("deterministic")
    }
}

impl<S: AgentState> CollapseAdapter<S> for WeightedCollapse {
    fn collapse(&self, state: &mut S, deltas: &[Box<dyn DeltaState + Send>]) -> CollapseMetadata {
        self.apply_many(state, deltas);
        CollapseMetadata::new("weighted")
    }
}

impl<S: AgentState> CollapseAdapter<S> for MultiStageCollapse {
    fn collapse(&self, state: &mut S, deltas: &[Box<dyn DeltaState + Send>]) -> CollapseMetadata {
        self.apply_many(state, deltas);
        let mut meta = CollapseMetadata::new("multi_stage");
        meta.stages = self.stages.len();
        meta
    }
}


