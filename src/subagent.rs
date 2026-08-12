//! Subagent execution engine for dax_agent_orchestrator.
//!
//! Supports:
//! - Micro‑agent routing
//! - Fractal recursive execution
//! - Depth‑limited and cost‑limited recursion
//! - Deterministic parallel execution
//! - Rich provenance metadata

use crate::traits::{
    Agent, AgentExecutor, AgentState, DeltaState,
    FractalAgent, MicroAgent, SubAgentSpec, Task,
};
use std::collections::HashMap;
use std::sync::Arc;

// ============================================================================
// RESULT ENVELOPE
// ============================================================================

#[derive(Debug)]
pub struct SubAgentResult {
    pub id: String,
    pub delta: Box<dyn DeltaState + Send>,
    pub metadata: Option<HashMap<String, String>>,
}

// ============================================================================
// MICRO‑AGENT ROUTER (FIXED ARC SIGNATURE)
// ============================================================================

/// Route a task to the correct micro‑agent.
/// Returns None if no micro‑agent accepts the task.
pub fn route_micro_agent<S: AgentState>(
    agents: &[Arc<dyn Agent<S>>],
    task: &Task,
) -> Option<Arc<dyn MicroAgent<S>>> {
    for agent in agents {
        if let Some(micro_arc) = agent.as_micro() {
            if micro_arc.accepts(task) {
                return Some(micro_arc.clone());
            }
        }
    }
    None
}

// ============================================================================
// FRACTAL EXECUTION ENGINE
// ============================================================================

/// Recursively execute a fractal agent.
pub fn run_fractal_recursive<S: AgentState>(
    agent: &dyn FractalAgent<S>,
    state: S,
    depth: usize,
) -> Vec<SubAgentSpec<S>> {
    let cfg = agent.config();

    if depth >= cfg.max_depth {
        return vec![];
    }

    let cost = agent.estimate_cost(&state);
    if cost > cfg.max_cost {
        return vec![];
    }

    let subs = agent.split(state.clone(), depth);

    let mut expanded = Vec::new();
    for sub in subs {
        expanded.push(sub.clone());

        let child_subs =
            run_fractal_recursive(agent, sub.scoped_state.clone(), depth + 1);

        expanded.extend(child_subs);
    }

    expanded
}

// ============================================================================
// SYNC RUNNER (LOCAL)
// ============================================================================

pub fn run_subagents_local<S, E>(
    specs: Vec<SubAgentSpec<S>>,
    executor: &E,
) -> Vec<SubAgentResult>
where
    S: AgentState,
    E: AgentExecutor<S>,
{
    specs
        .into_iter()
        .map(|spec| {
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

// ============================================================================
// ASYNC PARALLEL RUNNER
// ============================================================================

pub async fn run_subagents_parallel<S, E>(
    specs: Vec<SubAgentSpec<S>>,
    executor: Arc<E>,
) -> Vec<SubAgentResult>
where
    S: AgentState + Send + 'static,
    E: AgentExecutor<S> + Send + Sync + 'static,
{
    #[cfg(feature = "with-async")]
    {
        use tokio::task;

        let mut handles = Vec::with_capacity(specs.len());
        for spec in specs {
            let exec = executor.clone();
            handles.push(task::spawn_blocking(move || {
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
            if let Ok(res) = h.await {
                results.push(res);
            }
        }
        return results;
    }

    #[cfg(not(feature = "with-async"))]
    {
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
        return results;
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::{AgentExecutor, AgentState, DeltaState, Task};

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

        let exec = Arc::new(TestExecutor);
        let results = run_subagents_parallel(specs, exec).await;

        let mut applied = SimpleState { counter: 0 };
        for r in results {
            applied.apply_delta(r.delta.as_ref());
        }
        assert_eq!(applied.counter, 10);
    }
}



