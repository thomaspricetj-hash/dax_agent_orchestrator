//! dax_agent_orchestrator
//!
//! Public API re-exports and feature-gated items for consumers and examples.

pub mod traits;
pub mod dax;
pub mod subagent;

/// Core trait and types
pub use crate::traits::{
    AgentState, DeltaState, SubAgentSpec, Task, AgentExecutor, AgentExecutorAsync, BoxDeltaFuture,
};

/// DAX helpers: split / collapse and advanced collapse helpers
pub use crate::dax::{
    split, collapse, collapse_with, collapse_from_id_pairs, SplitStrategy, CollapseStrategy,
};

/// Subagent runners and result envelope
pub use crate::subagent::{run_subagents_local, run_subagents_parallel, SubAgentResult};

/// Optional re-exports when serde feature is enabled
#[cfg(feature = "with-serde")]
pub use serde;
#[cfg(feature = "with-serde")]
pub use serde_json;
