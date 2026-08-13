//! Task system — MAX‑TIER
//!
//! Defines:
//! - TaskKind (classification of tasks)
//! - TaskMetadata (provenance + routing info)
//! - Task (core unit of work)

use std::fmt::Debug;

// ============================================================================
// TASK KIND
// ============================================================================

#[derive(Clone, Debug)]
pub enum TaskKind {
    /// Generic task
    Generic,

    /// Micro‑agent specific task
    Micro,

    /// Fractal recursive task
    Fractal,

    /// Reflection‑driven task
    Reflective,

    /// Collapse‑driven task
    Collapse,

    /// Merge‑driven task
    Merge,

    /// System / internal task
    System,
}

// ============================================================================
// TASK METADATA
// ============================================================================

#[derive(Clone, Debug)]
pub struct TaskMetadata {
    pub source: Option<String>,
    pub description: Option<String>,
    pub timestamp: Option<u64>,
    pub priority: f32,
}

impl TaskMetadata {
    pub fn new() -> Self {
        Self {
            source: None,
            description: None,
            timestamp: None,
            priority: 1.0,
        }
    }

    pub fn with_priority(mut self, p: f32) -> Self {
        self.priority = p;
        self
    }
}

// ============================================================================
// TASK STRUCTURE
// ============================================================================

#[derive(Clone, Debug)]
pub struct Task {
    pub name: String,
    pub kind: TaskKind,
    pub metadata: TaskMetadata,
}

impl Task {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: TaskKind::Generic,
            metadata: TaskMetadata::new(),
        }
    }

    pub fn with_kind(mut self, kind: TaskKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn with_metadata(mut self, meta: TaskMetadata) -> Self {
        self.metadata = meta;
        self
    }
}
