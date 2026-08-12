DAX Agent Orchestrator – Install & Integration Guide

Overview
A concise, practical guide to installing and integrating the DAX Agent Orchestrator into an existing Rust agent. This covers dependency setup, feature flags, required trait implementations, example usage for splitting/executing/collapsing, testing, CI recommendations, and common troubleshooting steps.

Add dependency

Step 1 — Add the crate to your Cargo.toml

If using crates.io:

[dependencies]
dax_agent_orchestrator = "0.2"

If developing locally and the orchestrator is in a sibling folder:

[dependencies]
dax_agent_orchestrator = { path = "../dax_agent_orchestrator" }

Step 2 — Enable features you need

Common features:

with-async   Enables Tokio-backed parallel runner and async helpers.
with-serde   Enables optional JSON payload support in Task.

Example enabling both:

[dependencies.dax_agent_orchestrator]
version = "0.2"
features = ["with-async", "with-serde"]

Feature flags and runtime requirements

with-async
Use this when your host uses Tokio or you want the Tokio-backed run_subagents_parallel.

Your binary must depend on Tokio:

tokio = { version = "1", features = ["rt-multi-thread", "macros"] }

If you do not enable with-async, the library falls back to a thread-based parallel runner.

with-serde
Enables Task structured payload support.

Add serde if you plan to use JSON payloads:

serde = { version = "1", features = ["derive"] }

Example host binary Cargo.toml:

[dependencies]
dax_agent_orchestrator = { version = "0.2", features = ["with-async", "with-serde"] }
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
serde = { version = "1", features = ["derive"] }

Implementing the required traits

You must implement AgentState, define concrete Delta types, and implement an executor.

Implement AgentState

use dax_agent_orchestrator::traits::{AgentState, DeltaState};

#[derive(Clone, Debug)]
struct MyState { /* fields */ }

impl AgentState for MyState {
fn apply_delta(&mut self, delta: &dyn DeltaState) {
if let Some(d) = delta.as_any().downcast_ref::<MyDelta>() {
self.counter += d.delta;
} else {
// handle unknown delta types
}
}
}

Define concrete delta types

#[derive(Debug)]
struct MyDelta { delta: i64 }

This automatically implements DeltaState because it satisfies Send + Debug + Any + 'static.

Implement an executor

Synchronous executor:

use dax_agent_orchestrator::traits::{AgentExecutor, Task};

struct MyExecutor;

impl AgentExecutor<MyState> for MyExecutor {
fn run(&self, state: MyState, task: Task) -> Box<dyn DeltaState + Send> {
Box::new(MyDelta { delta: 1 })
}
}

Async executor (optional):

use dax_agent_orchestrator::traits::AgentExecutorAsync;
use std::pin::Pin;
use std::future::Future;

struct MyAsyncExecutor;

impl AgentExecutorAsync<MyState> for MyAsyncExecutor {
type Fut = Pin<Box<dyn Future<Output = Box<dyn DeltaState + Send>> + Send>>;

fn run_async(&self, state: MyState, task: Task) -> Self::Fut {
Box::pin(async move {
Box::new(MyDelta { delta: 1 })
})
}
}

Running the split → execute → collapse flow

Split into SubAgentSpecs

use dax_agent_orchestrator::{split, SplitStrategy, SubAgentSpec, Task};

let master: MyState = /* ... */;
let tasks = vec![
Task::new("t1", "payload1"),
Task::new("t2", "payload2"),
];

let specs: Vec<SubAgentSpec<MyState>> =
split(&master, SplitStrategy::SemanticRouting, tasks, |s, _i| s.clone());

Run subagents synchronously

use dax_agent_orchestrator::run_subagents_local;

let exec = MyExecutor;
let results = run_subagents_local(specs, &exec);

Run subagents in parallel (Tokio)

use dax_agent_orchestrator::run_subagents_parallel;
use std::sync::Arc;

let exec_arc = Arc::new(MyExecutor);
let results = run_subagents_parallel(specs, exec_arc).await;

Collapse deltas back into master

use dax_agent_orchestrator::{collapse_from_id_pairs, CollapseStrategy};

let id_and_deltas: Vec<(String, Box<dyn DeltaState + Send>)> =
results.into_iter().map(|r| (r.id, r.delta)).collect();

let new_master =
collapse_from_id_pairs(master, id_and_deltas, CollapseStrategy::Sequential);

Custom merge logic

use dax_agent_orchestrator::collapse_with;

let merged = collapse_with(master, deltas_with_ids, |master, delta, id_opt| {
// custom merge logic using id_opt and delta downcast
});

Testing, CI, and troubleshooting

Run tests locally:

cargo test

Run async tests:

cargo test --features "with-async"

Run ignored tests:

cargo test -- --ignored

Run doctests:

cargo test --doc

CI recommendations:

Matrix builds:
cargo test
cargo test --features "with-async"
cargo test --features "with-serde"

Linting:
cargo clippy --all-targets --all-features -- -D warnings

Format check:
cargo fmt -- --check

Common troubleshooting

Downcast returns None:
Ensure the delta type matches the executor’s output and implements Send + Debug + Any + 'static.

Feature-gated tests skipped:
Enable required features on the cargo test command line.

Tokio runtime errors:
Ensure your binary includes Tokio and with-async is enabled.

Unused import warnings:
Conditionally import async helpers using #[cfg(feature = "with-async")].

Panics in tests:
Inspect unwrap() calls; prefer graceful error handling or assert messages.

