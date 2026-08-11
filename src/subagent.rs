use crate::traits::{AgentState, DeltaState, SubAgentSpec};
use std::collections::HashMap;
use std::sync::Arc;

/// Result envelope for a subagent run.
#[derive(Debug)]
pub struct SubAgentResult {
    /// Unique subagent id (from the spec).
    pub id: String,
    /// Delta produced by the subagent.
    pub delta: Box<dyn DeltaState + Send>,
    /// Optional provenance/metadata (latency, executor id, confidence, etc.).
    pub metadata: Option<HashMap<String, String>>,
}

/// Lightweight helper to run subagents synchronously using the host executor.
/// Hosts can ignore this and run subagents in their own runtime or in parallel.
pub fn run_subagents_local<S, E>(specs: Vec<SubAgentSpec<S>>, executor: &E) -> Vec<SubAgentResult>
where
    S: AgentState,
    E: crate::traits::AgentExecutor<S>,
{
    specs
        .into_iter()
        .map(|spec| {
            // Simple provenance example: mark that this delta was produced by the local executor.
            let mut meta = HashMap::new();
            meta.insert("executor".to_string(), "local".to_string());
            let delta = executor.run(spec.scoped_state, spec.task.clone());
            SubAgentResult {
                id: spec.id,
                delta,
                metadata: Some(meta),
            }
        })
        .collect()
}

/// Async parallel runner that returns deltas in the same order as specs.
///
/// Behavior:
/// - When the `with-async` feature is enabled, this uses `tokio::task::spawn_blocking`
///   to run potentially blocking host executors inside a Tokio runtime and awaits
///   all spawned tasks concurrently.
/// - When the `with-async` feature is **not** enabled, it falls back to a
///   thread-based implementation using `std::thread::spawn`.
///
/// The function is `async` so callers can `.await` it uniformly.
pub async fn run_subagents_parallel<S, E>(
    specs: Vec<SubAgentSpec<S>>,
    executor: Arc<E>,
) -> Vec<SubAgentResult>
where
    S: AgentState + Send + 'static,
    E: crate::traits::AgentExecutor<S> + Send + Sync + 'static,
{
    // Tokio-backed implementation (preferred when feature enabled).
    #[cfg(feature = "with-async")]
    {
        use tokio::task;

        // Spawn tasks in the same order as specs; collect JoinHandles in order so awaiting
        // them in sequence preserves deterministic ordering of results.
        let mut handles = Vec::with_capacity(specs.len());
        for spec in specs {
            let exec = executor.clone();
            handles.push(task::spawn_blocking(move || {
                // Hosts may include richer metadata by returning it inside the DeltaState
                // or via other channels; here we keep metadata None for spawned tasks.
                let delta = exec.run(spec.scoped_state, spec.task.clone());
                SubAgentResult {
                    id: spec.id,
                    delta,
                    metadata: None,
                }
            }));
        }

        let mut results = Vec::with_capacity(handles.len());
        for h in handles {
            // If a task panics or the join fails, we skip that result.
            if let Ok(res) = h.await {
                results.push(res);
            }
        }
        results
    }

    // Fallback thread-based implementation when Tokio is not enabled.
    #[cfg(not(feature = "with-async"))]
    {
        // Use std threads to run the executor in parallel. This is a simple fallback
        // and will block the current thread while joining; it's suitable for small
        // workloads or environments without Tokio.
        let mut handles = Vec::with_capacity(specs.len());
        for spec in specs {
            let exec = executor.clone();
            handles.push(std::thread::spawn(move || {
                let delta = exec.run(spec.scoped_state, spec.task.clone());
                SubAgentResult {
                    id: spec.id,
                    delta,
                    metadata: None,
                }
            }));
        }

        let mut results = Vec::with_capacity(handles.len());
        for h in handles {
            if let Ok(res) = h.join() {
                results.push(res);
            }
        }
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::{AgentExecutor, AgentState, DeltaState, Task};

    // Only import Arc when the async feature is enabled and the parallel test is compiled.
    #[cfg(feature = "with-async")]
    use std::sync::Arc;

    #[derive(Clone, Debug)]
    struct SimpleState {
        pub counter: i64,
    }

    impl AgentState for SimpleState {
        fn apply_delta(&mut self, delta: &dyn DeltaState) {
            if let Some(d) = delta.as_any().downcast_ref::<SimpleDelta>() {
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

    impl Into<Box<dyn DeltaState + Send>> for SimpleDelta {
        fn into(self) -> Box<dyn DeltaState + Send> {
            Box::new(self)
        }
    }

    struct TestExecutor;
    impl AgentExecutor<SimpleState> for TestExecutor {
        fn run(&self, _state: SimpleState, task: Task) -> Box<dyn DeltaState + Send> {
            let inc = task.payload.parse::<i64>().unwrap_or(1);
            Box::new(SimpleDelta { delta: inc })
        }
    }

    #[test]
    fn local_runner_applies_all() {
        let master = SimpleState { counter: 0 };
        let tasks = vec![
            Task::new("A", "3"),
            Task::new("B", "5"),
            Task::new("C", "2"),
        ];
        let specs = tasks
            .into_iter()
            .enumerate()
            .map(|(i, t)| SubAgentSpec {
                id: format!("sub-{}", i),
                scoped_state: master.clone(),
                task: t,
            })
            .collect();

        let exec = TestExecutor;
        let results = run_subagents_local(specs, &exec);
        assert_eq!(results.len(), 3);

        // Apply each delta to a fresh master using the host's apply_delta implementation.
        let mut applied = SimpleState { counter: 0 };
        for r in results {
            applied.apply_delta(r.delta.as_ref());
        }
        assert_eq!(applied.counter, 10);
    }

    #[cfg(feature = "with-async")]
    #[tokio::test(flavor = "multi_thread")]
    async fn parallel_runner_tokio() {
        let master = SimpleState { counter: 0 };
        let tasks = vec![
            Task::new("A", "3"),
            Task::new("B", "5"),
            Task::new("C", "2"),
        ];
        let specs = tasks
            .into_iter()
            .enumerate()
            .map(|(i, t)| SubAgentSpec {
                id: format!("sub-{}", i),
                scoped_state: master.clone(),
                task: t,
            })
            .collect();

        // Use Arc here so the earlier unconditional import is actually used.
        let exec = Arc::new(TestExecutor);
        let results = run_subagents_parallel(specs, exec).await;
        assert_eq!(results.len(), 3);

        // Apply each delta to a fresh master using the host's apply_delta implementation.
        let mut applied = SimpleState { counter: 0 };
        for r in results {
            applied.apply_delta(r.delta.as_ref());
        }
        assert_eq!(applied.counter, 10);
    }
}



