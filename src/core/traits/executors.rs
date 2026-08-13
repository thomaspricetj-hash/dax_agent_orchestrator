//! Agent executors — MAX‑TIER
//!
//! Defines:
//! - AgentExecutor (core execution trait)
//! - LocalExecutor (single‑threaded execution)
//! - ParallelExecutor (multi‑threaded execution)

use std::fmt::Debug;
use std::sync::Arc;

use super::agent_state::AgentState;
use super::task::Task;
use super::delta::DeltaState;

// ============================================================================
// AGENT EXECUTOR TRAIT
// ============================================================================

/// Core executor trait.
/// Runs a task against a scoped agent state and produces a delta.
pub trait AgentExecutor<S: AgentState>: Send + Sync {
    fn run(
        &self,
        state: S,
        task: Task,
    ) -> Box<dyn DeltaState + Send>;
}

// ============================================================================
// LOCAL EXECUTOR (SYNC)
// ============================================================================

#[derive(Debug)]
pub struct LocalExecutor<E> {
    pub inner: Arc<E>,
}

impl<E> LocalExecutor<E> {
    pub fn new(inner: Arc<E>) -> Self {
        Self { inner }
    }
}

impl<S, E> AgentExecutor<S> for LocalExecutor<E>
where
    S: AgentState,
    E: AgentExecutor<S> + Send + Sync + 'static,
{
    fn run(
        &self,
        state: S,
        task: Task,
    ) -> Box<dyn DeltaState + Send> {
        self.inner.run(state, task)
    }
}

// ============================================================================
// PARALLEL EXECUTOR (THREAD SPAWN)
// ============================================================================

#[derive(Debug)]
pub struct ParallelExecutor<E> {
    pub inner: Arc<E>,
}

impl<E> ParallelExecutor<E> {
    pub fn new(inner: Arc<E>) -> Self {
        Self { inner }
    }
}

impl<S, E> AgentExecutor<S> for ParallelExecutor<E>
where
    S: AgentState + Send + 'static,
    E: AgentExecutor<S> + Send + Sync + 'static,
{
    fn run(
        &self,
        state: S,
        task: Task,
    ) -> Box<dyn DeltaState + Send> {
        let inner = self.inner.clone();
        let handle = std::thread::spawn(move || {
            inner.run(state, task)
        });

        handle.join().unwrap()
    }
}
