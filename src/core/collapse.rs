//! MAX‑TIER COLLAPSE ENGINE
//! Deterministic, weighted, and multi‑stage collapse over agent state.

use std::sync::Arc;

use crate::core::traits::{
    AgentState,
    CollapseStrategy,
    CollapseAdapter,
    CollapseMetadata,
    CollapseStage,
    DeltaState,
};

/// High‑level collapse mode.
#[derive(Clone, Debug)]
pub enum CollapseMode {
    Deterministic,
    Weighted,
    MultiStage(Vec<CollapseStage>),
}

/// Configuration for a collapse run.
#[derive(Clone, Debug)]
pub struct CollapseConfig {
    pub mode: CollapseMode,
}

impl CollapseConfig {
    pub fn deterministic() -> Self {
        Self {
            mode: CollapseMode::Deterministic,
        }
    }

    pub fn weighted() -> Self {
        Self {
            mode: CollapseMode::Weighted,
        }
    }

    pub fn multi_stage(stages: Vec<CollapseStage>) -> Self {
        Self {
            mode: CollapseMode::MultiStage(stages),
        }
    }
}

/// Concrete collapse engine that selects a strategy based on config.
///
/// Note: strategies are stored behind `Arc<dyn CollapseStrategy<S> + Send + Sync>`
/// so they can be cheaply cloned and reused without requiring `Clone` on the trait object.
pub struct CollapseEngine<S: AgentState> {
    pub deterministic: Arc<dyn CollapseStrategy<S> + Send + Sync>,
    pub weighted: Arc<dyn CollapseStrategy<S> + Send + Sync>,
}

impl<S: AgentState> CollapseEngine<S> {
    pub fn new(
        deterministic: Arc<dyn CollapseStrategy<S> + Send + Sync>,
        weighted: Arc<dyn CollapseStrategy<S> + Send + Sync>,
    ) -> Self {
        Self {
            deterministic,
            weighted,
        }
    }

    fn strategy_for(&self, config: &CollapseConfig) -> Arc<dyn CollapseStrategy<S> + Send + Sync> {
        match &config.mode {
            CollapseMode::Deterministic => Arc::clone(&self.deterministic),
            CollapseMode::Weighted => Arc::clone(&self.weighted),
            CollapseMode::MultiStage(stages) => {
                Arc::new(crate::core::traits::MultiStageCollapse::new(stages.clone()))
            }
        }
    }

    pub fn collapse(
        &self,
        config: &CollapseConfig,
        state: &mut S,
        deltas: &[Box<dyn DeltaState + Send>],
    ) -> CollapseMetadata {
        let strategy = self.strategy_for(config);
        strategy.apply_many(state, deltas);
        strategy.metadata().unwrap_or_else(|| CollapseMetadata::new("unknown"))
    }
}

/// Adapter that wraps the engine behind the CollapseAdapter trait.
pub struct EngineCollapseAdapter<S: AgentState> {
    pub engine: CollapseEngine<S>,
    pub config: CollapseConfig,
}

impl<S: AgentState> EngineCollapseAdapter<S> {
    pub fn new(engine: CollapseEngine<S>, config: CollapseConfig) -> Self {
        Self { engine, config }
    }
}

impl<S: AgentState> CollapseAdapter<S> for EngineCollapseAdapter<S> {
    fn collapse(
        &self,
        state: &mut S,
        deltas: &[Box<dyn DeltaState + Send>],
    ) -> CollapseMetadata {
        self.engine.collapse(&self.config, state, deltas)
    }
}
