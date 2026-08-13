//! SyntheticMind MAX‑TIER Cognitive Engine
//!
//! Public API surface for the full agent orchestration system.
//!
//! Exposes:
//! - Core trait system
//! - Delta system
//! - Merge engine
//! - Collapse engine
//! - Agent tree + fractal recursion
//! - DAX orchestrator
//! - Subagent execution engine

pub mod core;
pub mod engine;

// ============================================================================
// CORE RE‑EXPORTS
// ============================================================================

// All traits (agent, micro, fractal, reflection, collapse, merge, cost, etc.)
pub use core::traits::*;

// Delta system
#[allow(unused_imports)]
pub use core::traits::delta::*;

// Merge engine
#[allow(unused_imports)]
pub use core::traits::merge::*;

// Collapse engine
#[allow(unused_imports)]
pub use core::traits::collapse::*;

// Agent tree
pub use core::agent_tree::*;

// ============================================================================
// ENGINE RE‑EXPORTS
// ============================================================================

// DAX orchestrator
pub use engine::dax::*;

// Subagent execution engine
pub use engine::subagent::*;

