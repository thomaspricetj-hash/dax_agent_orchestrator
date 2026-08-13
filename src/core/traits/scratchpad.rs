//! Scratchpad system — MAX‑TIER
//!
//! Defines:
//! - ScratchpadEntry (single note/fact)
//! - Scratchpad (collection of entries)
//! - ScratchpadAgent (agents with internal scratchpad memory)

use std::fmt::Debug;

use super::agent_state::AgentState;
use super::task::Task;

// ============================================================================
// SCRATCHPAD ENTRY
// ============================================================================

#[derive(Clone, Debug)]
pub struct ScratchpadEntry {
    pub key: String,
    pub value: String,
    pub timestamp: u64,
}

impl ScratchpadEntry {
    pub fn new(key: impl Into<String>, value: impl Into<String>, timestamp: u64) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            timestamp,
        }
    }
}

// ============================================================================
// SCRATCHPAD STRUCTURE
// ============================================================================

#[derive(Clone, Debug)]
pub struct Scratchpad {
    pub entries: Vec<ScratchpadEntry>,
}

impl Scratchpad {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn add(&mut self, key: impl Into<String>, value: impl Into<String>, timestamp: u64) {
        self.entries.push(ScratchpadEntry::new(key, value, timestamp));
    }

    pub fn get(&self, key: &str) -> Option<&ScratchpadEntry> {
        self.entries.iter().find(|e| e.key == key)
    }
}

// ============================================================================
// SCRATCHPAD AGENT TRAIT
// ============================================================================

pub trait ScratchpadAgent<S: AgentState>: Send + Sync {
    /// Whether this agent supports scratchpad.
    fn has_scratchpad(&self) -> bool {
        true
    }

    /// Access the scratchpad.
    fn scratchpad(&self) -> &Scratchpad;

    /// Mutate the scratchpad.
    fn scratchpad_mut(&mut self) -> &mut Scratchpad;

    /// Optional gating: should scratchpad be used?
    fn scratchpad_enabled(&self, _state: &S, _task: &Task) -> bool {
        true
    }
}
