//! Core traits and types for dax_agent_orchestrator.
//!
//! Upgraded to support micro‑agents, fractal agents, recursive agent trees,
//! and next‑generation delta/state merging. This file defines the public
//! contracts hosts implement to integrate with the orchestrator.

use std::any::Any;
use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

// ============================================================================
// AGENT STATE + DELTA
// ============================================================================

/// Opaque trait for the host agent's state.
/// Must support delta application and cloning for recursive fractal execution.
pub trait AgentState: Clone + Send + Debug + 'static {
    fn apply_delta(&mut self, delta: &dyn DeltaState);
}

/// Opaque trait for deltas produced by subagents.
/// Must support safe downcasting.
pub trait DeltaState: Send + Debug + Any + 'static {
    fn as_any(&self) -> &dyn Any;
}

impl dyn DeltaState {
    pub fn downcast_ref<T: 'static>(&self) -> Option<&T> {
        self.as_any().downcast_ref::<T>()
    }
}

impl<T: Send + Debug + Any + 'static> DeltaState for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Helper trait to convert concrete deltas into boxed trait objects.
pub trait IntoBoxedDelta {
    fn into_boxed(self) -> Box<dyn DeltaState + Send>
    where
        Self: Sized;
}

impl<T> IntoBoxedDelta for T
where
    T: DeltaState + Send + 'static,
{
    fn into_boxed(self) -> Box<dyn DeltaState + Send> {
        Box::new(self)
    }
}

// ============================================================================
// TASK + SUBAGENT SPEC
// ============================================================================

#[non_exhaustive]
#[derive(Clone, Debug)]
pub struct Task {
    pub name: String,
    pub payload: String,

    #[cfg(feature = "with-serde")]
    pub payload_json: Option<serde_json::Value>,
}

impl Task {
    pub fn new<N, P>(name: N, payload: P) -> Self
    where
        N: Into<String>,
        P: Into<String>,
    {
        Task {
            name: name.into(),
            payload: payload.into(),
            #[cfg(feature = "with-serde")]
            payload_json: None,
        }
    }

    #[cfg(feature = "with-serde")]
    pub fn with_payload_json(mut self, value: serde_json::Value) -> Self {
        self.payload_json = Some(value);
        self
    }
}

/// Specification for a subagent produced by split().
#[derive(Clone, Debug)]
pub struct SubAgentSpec<S: AgentState> {
    pub id: String,
    pub scoped_state: S,
    pub task: Task,
}

// ============================================================================
// EXECUTORS (SYNC + ASYNC)
// ============================================================================

pub trait AgentExecutor<S: AgentState> {
    fn run(&self, state: S, task: Task) -> Box<dyn DeltaState + Send>;
}

pub trait AgentExecutorAsync<S: AgentState> {
    type Fut: Future<Output = Box<dyn DeltaState + Send>> + Send + 'static;
    fn run_async(&self, state: S, task: Task) -> Self::Fut;
}

pub type BoxDeltaFuture =
    Pin<Box<dyn Future<Output = Box<dyn DeltaState + Send>> + Send>>;

// ============================================================================
// MICRO‑AGENTS
// ============================================================================

/// Micro‑agents are atomic cognitive units. They perform one small step.
pub trait MicroAgent<S: AgentState>: Send + Sync + Debug {
    fn accepts(&self, task: &Task) -> bool;
    fn execute(&self, state: S, task: Task) -> Box<dyn DeltaState + Send>;
}

// ============================================================================
// FRACTAL AGENTS
// ============================================================================

#[derive(Clone, Debug)]
pub struct FractalConfig {
    pub max_depth: usize,
    pub max_cost: usize,
}

#[derive(Clone, Debug)]
pub struct AgentTree<S: AgentState> {
    pub depth: usize,
    pub cost: usize,
    pub state: S,
    pub children: Vec<SubAgentSpec<S>>,
}

pub trait FractalAgent<S: AgentState>: Send + Sync + Debug {
    fn split(&self, state: S, depth: usize) -> Vec<SubAgentSpec<S>>;

    fn estimate_cost(&self, _state: &S) -> usize {
        1
    }

    fn config(&self) -> FractalConfig {
        FractalConfig {
            max_depth: 3,
            max_cost: 128,
        }
    }
}

// ============================================================================
// UNIFIED AGENT TRAIT (FIXED ARC SIGNATURES)
// ============================================================================

pub trait Agent<S: AgentState>: Send + Sync + Debug + 'static {
    fn as_micro(&self) -> Option<Arc<dyn MicroAgent<S>>> {
        None
    }

    fn as_fractal(&self) -> Option<Arc<dyn FractalAgent<S>>> {
        None
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug)]
    struct SimpleState {
        pub counter: i64,
    }

    impl AgentState for SimpleState {
        fn apply_delta(&mut self, delta: &dyn DeltaState) {
            if let Some(d) = delta.downcast_ref::<SimpleDelta>() {
                self.counter += d.delta;
            } else {
                panic!("unexpected delta type");
            }
        }
    }

    #[derive(Debug)]
    struct SimpleDelta {
        delta: i64,
    }

    #[test]
    fn downcast_and_apply_delta() {
        let delta = SimpleDelta { delta: 7 };
        let boxed: Box<dyn DeltaState + Send> = delta.into_boxed();

        assert_eq!(
            boxed.as_ref().as_any().downcast_ref::<SimpleDelta>().unwrap().delta,
            7
        );

        let mut state = SimpleState { counter: 3 };
        state.apply_delta(boxed.as_ref());
        assert_eq!(state.counter, 10);
    }

    #[test]
    fn task_constructor() {
        let t = Task::new("task1", "payload");
        assert_eq!(t.name, "task1");
        assert_eq!(t.payload, "payload");
    }
}

