//! Merge system — MAX‑TIER
//!
//! Defines:
//! - MergeMetadata
//! - MergeStrategy
//! - DeterministicMerge
//! - WeightedMerge
//! - MultiAgentMerge
//! - MergeAdapter

use std::fmt;
use std::sync::Arc;

use super::delta::{DeltaState, ZeroDelta};

// ============================================================================
// MERGE METADATA
// ============================================================================

#[derive(Clone, Debug)]
pub struct MergeMetadata {
    pub strategy: String,
    pub agent_count: usize,
    pub delta_count: usize,
    pub total_weight: f32,
    pub provenance: Option<String>,
}

impl MergeMetadata {
    pub fn new(strategy: &str) -> Self {
        Self {
            strategy: strategy.to_string(),
            agent_count: 0,
            delta_count: 0,
            total_weight: 1.0,
            provenance: None,
        }
    }
}

// ============================================================================
// MERGE STRATEGY TRAIT
// ============================================================================

/// Object‑safe merge strategy trait.
pub trait MergeStrategy: Send + Sync {
    /// Merge a slice of deltas into a single boxed delta.
    fn merge(&self, deltas: &[Box<dyn DeltaState + Send>]) -> Box<dyn DeltaState + Send>;

    /// Optional metadata about the merge run.
    fn metadata(&self) -> Option<MergeMetadata> {
        None
    }

    /// Optional prediction about the merge outcome.
    fn predict(&self, _deltas: &[Box<dyn DeltaState + Send>]) -> Option<String> {
        None
    }
}

// ============================================================================
// DETERMINISTIC MERGE
// ============================================================================

#[derive(Debug)]
pub struct DeterministicMerge;

impl MergeStrategy for DeterministicMerge {
    fn merge(&self, deltas: &[Box<dyn DeltaState + Send>]) -> Box<dyn DeltaState + Send> {
        if deltas.is_empty() {
            Box::new(ZeroDelta)
        } else {
            // `Box<dyn DeltaState + Send>` implements Clone via `DeltaState::clone_box`.
            deltas[0].clone()
        }
    }

    fn metadata(&self) -> Option<MergeMetadata> {
        Some(MergeMetadata::new("deterministic"))
    }
}

// ============================================================================
// WEIGHTED MERGE
// ============================================================================

#[derive(Debug)]
pub struct WeightedMerge;

impl MergeStrategy for WeightedMerge {
    fn merge(&self, deltas: &[Box<dyn DeltaState + Send>]) -> Box<dyn DeltaState + Send> {
        if deltas.is_empty() {
            return Box::new(ZeroDelta);
        }

        let mut best: Option<(f32, usize)> = None;

        for (i, d) in deltas.iter().enumerate() {
            // Use the safe DeltaState::weight() accessor.
            let weight = d.weight().unwrap_or(1.0);

            match best {
                None => best = Some((weight, i)),
                Some((bw, _)) if weight > bw => best = Some((weight, i)),
                _ => {}
            }
        }

        let (_, idx) = best.unwrap();
        deltas[idx].clone()
    }

    fn metadata(&self) -> Option<MergeMetadata> {
        Some(MergeMetadata::new("weighted"))
    }
}

// ============================================================================
// MULTI‑AGENT MERGE
// ============================================================================

pub struct MultiAgentMerge {
    pub strategy: Arc<dyn MergeStrategy + Send + Sync>,
}

impl MultiAgentMerge {
    pub fn new(strategy: Arc<dyn MergeStrategy + Send + Sync>) -> Self {
        Self { strategy }
    }
}

impl MergeStrategy for MultiAgentMerge {
    fn merge(&self, deltas: &[Box<dyn DeltaState + Send>]) -> Box<dyn DeltaState + Send> {
        self.strategy.merge(deltas)
    }

    fn metadata(&self) -> Option<MergeMetadata> {
        let mut meta = MergeMetadata::new("multi_agent");
        meta.agent_count = 1; // placeholder
        meta.delta_count = 1; // placeholder
        Some(meta)
    }
}

impl fmt::Debug for MultiAgentMerge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MultiAgentMerge")
            .field("strategy", &"<dyn MergeStrategy>")
            .finish()
    }
}

// ============================================================================
// MERGE ADAPTER
// ============================================================================

/// Adapter trait that returns both merged delta and metadata.
pub trait MergeAdapter: Send + Sync {
    fn merge(&self, deltas: &[Box<dyn DeltaState + Send>]) -> (Box<dyn DeltaState + Send>, MergeMetadata);
}

impl<T: MergeStrategy + Send + Sync> MergeAdapter for T {
    fn merge(&self, deltas: &[Box<dyn DeltaState + Send>]) -> (Box<dyn DeltaState + Send>, MergeMetadata) {
        // Call the trait method explicitly to avoid accidental name shadowing.
        let merged = MergeStrategy::merge(self, deltas);
        let meta = self.metadata().unwrap_or_else(|| MergeMetadata::new("unknown"));
        (merged, meta)
    }
}
