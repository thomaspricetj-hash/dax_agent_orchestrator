//! Core traits and types for dax_agent_orchestrator.
//!
//! This file contains the public traits that hosts implement to integrate
//! with the orchestrator: `AgentState`, `DeltaState`, `AgentExecutor` and
//! `AgentExecutorAsync`. It also provides ergonomic helpers and a small test
//! suite to validate downcasting and delta application.

use std::any::Any;
use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;

/// Opaque trait for the host agent's state.
/// Host agents implement this for their own memory representation.
///
/// Implementers must ensure `apply_delta` correctly merges or applies
/// the provided `DeltaState` into the concrete state representation.
pub trait AgentState: Clone + Send + Debug + 'static {
    /// Apply a delta produced by a subagent to this state in-place.
    fn apply_delta(&mut self, delta: &dyn DeltaState);
}

/// Opaque trait for deltas produced by subagents.
/// Host agents define concrete delta types and implement this trait.
///
/// `as_any` is provided to allow safe downcasting from trait objects.
/// The short illustrative examples in the docs are intentionally marked
/// `ignore` so they don't run as doctests; replace with real examples
/// in your host crate when integrating.
pub trait DeltaState: Send + Debug + Any + 'static {
    /// Expose a reference as `Any` for safe downcasting.
    fn as_any(&self) -> &dyn Any;
}

/// Provide a convenient downcast helper as an inherent method on the trait object.
/// This does not make the trait non-object-safe because the generic method is not
/// part of the trait definition itself.
impl dyn DeltaState {
    /// Downcast a `dyn DeltaState` reference to a concrete type `T`.
    pub fn downcast_ref<T: 'static>(&self) -> Option<&T> {
        self.as_any().downcast_ref::<T>()
    }
}

/// Blanket impl so concrete delta types automatically implement `DeltaState`.
impl<T: Send + Debug + Any + 'static> DeltaState for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Helper trait to convert concrete delta values into boxed trait objects.
/// This reduces boilerplate in executors that return concrete delta types.
pub trait IntoBoxedDelta {
    /// Convert `self` into a boxed `DeltaState`.
    fn into_boxed(self) -> Box<dyn DeltaState + Send>
    where
        Self: Sized;
}

impl<T> IntoBoxedDelta for T
where
    T: DeltaState + Send + 'static,
{
    fn into_boxed(self) -> Box<dyn DeltaState + Send>
    where
        Self: Sized,
    {
        Box::new(self)
    }
}

/// Task descriptor passed to subagents.
///
/// `payload` is a simple string for hosts that prefer text payloads.
/// If you enable the `with-serde` feature, hosts may also populate `payload_json`.
#[non_exhaustive]
#[derive(Clone, Debug)]
pub struct Task {
    /// Short human-readable name for the task (used for logging/routing).
    pub name: String,
    /// Opaque string payload. Hosts may use JSON, protobuf, or any encoding here.
    pub payload: String,
    /// Optional structured payload available when the host enables `with-serde`.
    #[cfg(feature = "with-serde")]
    pub payload_json: Option<serde_json::Value>,
}

impl Task {
    /// Create a new `Task` with a name and string payload.
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

    /// Set the structured JSON payload (only available when `with-serde` is enabled).
    #[cfg(feature = "with-serde")]
    pub fn with_payload_json(mut self, value: serde_json::Value) -> Self {
        self.payload_json = Some(value);
        self
    }
}

/// Specification for a subagent produced by `split`.
#[derive(Clone, Debug)]
pub struct SubAgentSpec<S: AgentState> {
    /// Unique id for the subagent spec (e.g., "sub-0", "reasoning::1").
    pub id: String,
    /// Scoped copy or view of the host's state for the subagent to operate on.
    pub scoped_state: S,
    /// Task the subagent should perform.
    pub task: Task,
}

/// Executor trait the host agent implements to run a subagent synchronously.
///
/// The orchestrator will call this to execute subagents. Hosts return a boxed
/// `DeltaState` that the orchestrator will pass back into `collapse`.
pub trait AgentExecutor<S: AgentState> {
    /// Run a subagent given its scoped state and task.
    /// Return a boxed `DeltaState` (host-defined concrete type).
    fn run(&self, state: S, task: Task) -> Box<dyn DeltaState + Send>;
}

/// Async executor trait for hosts that run subagents asynchronously.
///
/// This trait is optional; use it when your host runtime prefers async execution.
/// The returned future must be `Send` and resolve to a boxed `DeltaState`.
pub trait AgentExecutorAsync<S: AgentState> {
    /// Type alias for the boxed future that resolves to a boxed delta.
    type Fut: Future<Output = Box<dyn DeltaState + Send>> + Send + 'static;

    /// Run a subagent asynchronously and return a future that yields a delta.
    fn run_async(&self, state: S, task: Task) -> Self::Fut;
}

/// Convenience type for implementers who prefer a boxed future return type.
///
/// Example implementation using boxed futures:
/// ```ignore
/// use std::pin::Pin;
/// use std::future::Future;
///
/// // impl AgentExecutorAsync<MyState> for MyExecutor { ... }
/// ```
pub type BoxDeltaFuture = Pin<Box<dyn Future<Output = Box<dyn DeltaState + Send>> + Send>>;

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
        // test into_boxed helper
        let boxed: Box<dyn DeltaState + Send> = delta.into_boxed();

        // Use as_any() to downcast from the trait object reference.
        assert_eq!(
            boxed.as_ref().as_any().downcast_ref::<SimpleDelta>().unwrap().delta,
            7
        );

        // test apply_delta via AgentState
        let mut state = SimpleState { counter: 3 };
        state.apply_delta(boxed.as_ref());
        assert_eq!(state.counter, 10);
    }

    #[test]
    fn task_constructor_and_payload_json_feature() {
        let t = Task::new("task1", "payload");
        assert_eq!(t.name, "task1");
        assert_eq!(t.payload, "payload");
    }
}


