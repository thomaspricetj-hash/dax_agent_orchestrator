//! Agent state — MAX‑TIER foundation
//!
//! Defines:
//! - AgentState (host agent state container)
//! - Optional cost prediction
//! - Optional provenance metadata

use std::fmt::Debug;

use super::delta::DeltaState;

// ============================================================================
// AGENT STATE (MAX‑TIER)
// ============================================================================

/// Host agent state.
/// Stores the agent’s internal memory and applies deltas.
pub trait AgentState: Clone + Send + Sync + Debug + 'static {
    /// Apply a delta to the state.
    fn apply_delta(&mut self, delta: &dyn DeltaState);

    /// Optional: cost prediction hook.
    fn estimate_cost(&self) -> usize {
        1
    }

    /// Optional: provenance metadata.
    fn provenance(&self) -> Option<String> {
        None
    }
}
