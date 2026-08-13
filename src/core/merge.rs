//! MAX‑TIER MERGE ENGINE
//! Deterministic, weighted, and multi‑agent merge strategies.
//!
//! This module provides:
//! - MergeConfig
//! - MergeEngine
//! - DeterministicMergeEngine
//! - WeightedMergeEngine
//! - MultiAgentMergeEngine
//! - EngineMergeAdapter
//! - Delta grouping utilities

use std::sync::Arc;

use crate::core::traits::{
    DeltaState,
    MergeStrategy,
    MergeAdapter,
    MergeMetadata,
    DeterministicMerge,
    WeightedMerge,
    MultiAgentMerge,
};

/// Merge configuration.
#[derive(Clone, Debug)]
pub enum MergeMode {
    Deterministic,
    Weighted,
    MultiAgent,
}

/// Configuration wrapper.
#[derive(Clone, Debug)]
pub struct MergeConfig {
    pub mode: MergeMode,
}

impl MergeConfig {
    pub fn deterministic() -> Self {
        Self { mode: MergeMode::Deterministic }
    }

    pub fn weighted() -> Self {
        Self { mode: MergeMode::Weighted }
    }

    pub fn multi_agent() -> Self {
        Self { mode: MergeMode::MultiAgent }
    }
}

/// Concrete merge engine that selects a strategy based on config.
pub struct MergeEngine {
    pub deterministic: Arc<dyn MergeStrategy + Send + Sync>,
    pub weighted: Arc<dyn MergeStrategy + Send + Sync>,
    pub multi_agent: Arc<dyn MergeStrategy + Send + Sync>,
}

impl MergeEngine {
    pub fn new() -> Self {
        Self {
            deterministic: Arc::new(DeterministicMerge),
            weighted: Arc::new(WeightedMerge),
            multi_agent: Arc::new(MultiAgentMerge::new(Arc::new(WeightedMerge))),
        }
    }

    pub fn with_strategies(
        deterministic: Arc<dyn MergeStrategy + Send + Sync>,
        weighted: Arc<dyn MergeStrategy + Send + Sync>,
        multi_agent: Arc<dyn MergeStrategy + Send + Sync>,
    ) -> Self {
        Self {
            deterministic,
            weighted,
            multi_agent,
        }
    }

    fn strategy_for(&self, config: &MergeConfig) -> Arc<dyn MergeStrategy + Send + Sync> {
        match config.mode {
            MergeMode::Deterministic => self.deterministic.clone(),
            MergeMode::Weighted => self.weighted.clone(),
            MergeMode::MultiAgent => self.multi_agent.clone(),
        }
    }

    /// Merge deltas using the selected strategy.
    pub fn merge(
        &self,
        config: &MergeConfig,
        deltas: &[Box<dyn DeltaState + Send>],
    ) -> (Box<dyn DeltaState + Send>, MergeMetadata) {
        let strategy = self.strategy_for(config);
        let merged = strategy.merge(deltas);
        let meta = strategy.metadata().unwrap_or_else(|| MergeMetadata::new("unknown"));
        (merged, meta)
    }
}

/// Adapter that wraps the engine behind the MergeAdapter trait.
pub struct EngineMergeAdapter {
    pub engine: MergeEngine,
    pub config: MergeConfig,
}

impl EngineMergeAdapter {
    pub fn new(engine: MergeEngine, config: MergeConfig) -> Self {
        Self { engine, config }
    }
}

impl MergeAdapter for EngineMergeAdapter {
    fn merge(
        &self,
        deltas: &[Box<dyn DeltaState + Send>],
    ) -> (Box<dyn DeltaState + Send>, MergeMetadata) {
        self.engine.merge(&self.config, deltas)
    }
}

// ============================================================================
// DELTA GROUPING UTILITIES
// ============================================================================

/// Group descriptor that references deltas by index instead of owning them.
/// This avoids requiring `Clone` on `Box<dyn DeltaState + Send>`.
pub struct DeltaGroupRef {
    pub type_name: String,
    pub indices: Vec<usize>,
}

impl std::fmt::Debug for DeltaGroupRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeltaGroupRef")
            .field("type_name", &self.type_name)
            .field("count", &self.indices.len())
            .finish()
    }
}

/// Group deltas by type id string (useful for multi‑agent merge).
/// Returns groups that reference the original `deltas` slice by index.
pub fn group_deltas_by_type(
    deltas: &[Box<dyn DeltaState + Send>],
) -> Vec<DeltaGroupRef> {
    use std::collections::HashMap;

    let mut map: HashMap<String, Vec<usize>> = HashMap::new();

    for (i, d) in deltas.iter().enumerate() {
        // TypeId implements Debug but not Display. Use Debug formatting for a stable key.
        let type_name = format!("{:?}", d.as_any().type_id());
        map.entry(type_name).or_default().push(i);
    }

    map.into_iter()
        .map(|(type_name, indices)| DeltaGroupRef { type_name, indices })
        .collect()
}

// ============================================================================
// DELTA NORMALIZATION UTILITIES
// ============================================================================

/// Normalize delta weights so they sum to 1.0.
/// Uses the optional `DeltaState::weight()` accessor; concrete deltas should
/// override `weight()` to expose their weight. If a concrete delta supports
/// mutation, it can override `set_weight()` to accept the normalized value.
/// The default `set_weight()` is a no-op, so this function is safe for all deltas.
pub fn normalize_weights(deltas: &mut [Box<dyn DeltaState + Send>]) {
    // First pass: compute total weight for deltas that expose weight via DeltaState::weight()
    let mut total = 0.0f32;

    for d in deltas.iter() {
        // If a delta exposes a weight via the optional accessor, use it; otherwise default to 1.0
        let w = d.weight().unwrap_or(1.0);
        total += w;
    }

    if total <= 0.0 {
        return;
    }

    // Second pass: normalize weights and call the optional setter on each delta.
    for d in deltas.iter_mut() {
        let w = d.weight().unwrap_or(1.0);
        let new_w = w / total;
        // Call the trait-provided setter; concrete types that support mutation should override it.
        d.set_weight(new_w);
    }
}
