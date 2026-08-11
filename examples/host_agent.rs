// examples/host_agent.rs
// Run with:
//   cargo run --example host_agent --features "with-async"
// or without async:
//   cargo run --example host_agent

use dax_agent_orchestrator as orchestrator;
use orchestrator::{
    AgentState, DeltaState, Task, split, collapse_from_id_pairs, run_subagents_local,
    SplitStrategy, CollapseStrategy, AgentExecutor, SubAgentResult,
};
// Only import the parallel runner and Arc when async feature is enabled.
#[cfg(feature = "with-async")]
use orchestrator::run_subagents_parallel;
#[cfg(feature = "with-async")]
use std::sync::Arc;

use std::fmt::Debug;

/// Example concrete AgentState used by this host.
#[derive(Clone, Debug)]
struct SimpleState {
    pub counter: i64,
}

impl AgentState for SimpleState {
    fn apply_delta(&mut self, delta: &dyn DeltaState) {
        if let Some(d) = delta.as_any().downcast_ref::<SimpleDelta>() {
            self.counter += d.delta;
        } else {
            eprintln!("apply_delta: unknown delta type {:?}", delta);
        }
    }
}

/// Concrete DeltaState for this example.
#[derive(Debug)]
struct SimpleDelta {
    delta: i64,
}

/// Host executor that runs a subagent given scoped state and task.
struct HostExecutor;
impl AgentExecutor<SimpleState> for HostExecutor {
    fn run(&self, _state: SimpleState, task: Task) -> Box<dyn DeltaState + Send> {
        println!("Running subagent {} with payload {:?}", task.name, task.payload);
        let inc = task.payload.parse::<i64>().unwrap_or(1);
        Box::new(SimpleDelta { delta: inc })
    }
}

#[cfg(feature = "with-async")]
#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let master = SimpleState { counter: 0 };
    let tasks = vec![
        Task::new("A", "3"),
        Task::new("B", "5"),
        Task::new("C", "2"),
    ];

    // Create subagent specs by cloning the master state for each subagent.
    // Real hosts should provide semantic slicing here.
    let specs = split(&master, SplitStrategy::SemanticRouting, tasks, |s, _i| s.clone());

    // --- Synchronous run ---
    let executor = HostExecutor;
    let results: Vec<SubAgentResult> = run_subagents_local(specs.clone(), &executor);

    // Optionally inspect metadata or ids
    for r in &results {
        println!("Sync result id: {}, metadata: {:?}", r.id, r.metadata);
    }

    // Convert to (id, delta) pairs and collapse preserving ids
    let id_and_deltas_sync: Vec<(String, Box<dyn DeltaState + Send>)> =
        results.into_iter().map(|r| (r.id, r.delta)).collect();

    let new_master_sync = collapse_from_id_pairs(master.clone(), id_and_deltas_sync, CollapseStrategy::Sequential);
    println!("New master state (sync collapse): {:?}", new_master_sync);

    // --- Parallel run (Tokio-backed) ---
    // These imports are only present when the with-async feature is enabled,
    // so this code path is compiled only in that configuration.
    let executor_arc = Arc::new(HostExecutor);
    let results_par: Vec<SubAgentResult> = run_subagents_parallel(specs, executor_arc).await;

    for r in &results_par {
        println!("Parallel result id: {}, metadata: {:?}", r.id, r.metadata);
    }

    let id_and_deltas_par: Vec<(String, Box<dyn DeltaState + Send>)> =
        results_par.into_iter().map(|r| (r.id, r.delta)).collect();

    let new_master_par = collapse_from_id_pairs(master, id_and_deltas_par, CollapseStrategy::Sequential);
    println!("New master state (parallel collapse): {:?}", new_master_par);
}

#[cfg(not(feature = "with-async"))]
fn main() {
    // Non-async build: run only the synchronous runner to avoid requiring a runtime.
    let master = SimpleState { counter: 0 };
    let tasks = vec![
        Task::new("A", "3"),
        Task::new("B", "5"),
        Task::new("C", "2"),
    ];

    let specs = split(&master, SplitStrategy::SemanticRouting, tasks, |s, _i| s.clone());

    // Synchronous run
    let executor = HostExecutor;
    let results: Vec<SubAgentResult> = run_subagents_local(specs.clone(), &executor);

    for r in &results {
        println!("Sync result id: {}, metadata: {:?}", r.id, r.metadata);
    }

    let id_and_deltas_sync: Vec<(String, Box<dyn DeltaState + Send>)> =
        results.into_iter().map(|r| (r.id, r.delta)).collect();

    let new_master_sync = collapse_from_id_pairs(master, id_and_deltas_sync, CollapseStrategy::Sequential);
    println!("New master state (sync collapse): {:?}", new_master_sync);

    // Note: to exercise the parallel runner enable the with-async feature and run with Tokio.
}
