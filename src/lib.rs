//! dax_agent_orchestrator
//!
//! Public API re-exports for consumers, hosts, and examples.

pub mod traits;
pub mod dax;
pub mod subagent;

// ============================================================================
// CORE TRAITS + TYPES
// ============================================================================

pub use crate::traits::{
    AgentState,
    DeltaState,
    SubAgentSpec,
    Task,
    AgentExecutor,
    AgentExecutorAsync,
    BoxDeltaFuture,

    Agent,
    MicroAgent,
    FractalAgent,
    FractalConfig,
    AgentTree,
};

// ============================================================================
// DAX ORCHESTRATION PIPELINE (ONLY WHAT EXISTS IN THIS PROJECT)
// ============================================================================

pub use crate::dax::{
    SplitStrategy,
    CollapseStrategy,

    // These DO exist in your agent project's dax.rs
    dax_split,
    dax_split_fractal,
    dax_expand_fractal,
    dax_execute_sync,
    dax_execute_async,
    dax_collapse,
    dax_run_sync,
    dax_run_async,
};

// ============================================================================
// SUBAGENT RUNNERS
// ============================================================================

pub use crate::subagent::{
    run_subagents_local,
    run_subagents_parallel,
    SubAgentResult,
};

// ============================================================================
// OPTIONAL SERDE SUPPORT
// ============================================================================

#[cfg(feature = "with-serde")]
pub use serde;

#[cfg(feature = "with-serde")]
pub use serde_json;


