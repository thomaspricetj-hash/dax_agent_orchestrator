//! Capability introspection — MAX‑TIER
//!
//! Defines:
//! - AgentCapabilities (self‑describing capability flags)
//! - CapabilityIntrospection (agents report their capabilities)

use std::fmt::Debug;

use super::agent_state::AgentState;

// ============================================================================
// AGENT CAPABILITIES STRUCT
// ============================================================================

#[derive(Clone, Debug)]
pub struct AgentCapabilities {
    pub can_reflect: bool,
    pub can_fractal: bool,
    pub has_scratchpad: bool,
    pub has_dnd: bool,
    pub can_merge: bool,
    pub can_collapse: bool,
    pub can_predict_cost: bool,
}

impl AgentCapabilities {
    pub fn new() -> Self {
        Self {
            can_reflect: false,
            can_fractal: false,
            has_scratchpad: false,
            has_dnd: false,
            can_merge: false,
            can_collapse: false,
            can_predict_cost: false,
        }
    }
}

// ============================================================================
// CAPABILITY INTROSPECTION TRAIT
// ============================================================================

pub trait CapabilityIntrospection<S: AgentState>: Send + Sync {
    fn capabilities(&self) -> AgentCapabilities;
}
