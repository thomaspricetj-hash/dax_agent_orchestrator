Overview

A concise, practical guide to installing and integrating the DAX Agent Orchestrator into an existing Rust agent. This covers dependency setup, feature flags, the minimal trait implementations you must provide, example usage for splitting/executing/collapsing, running tests, CI recommendations, and common troubleshooting steps.



Add dependency

Step 1 — Add the crate to your Cargo.toml



If you publish your agent as a workspace member or use the crate from crates.io, add a dependency entry. Example for a crates.io release:



toml

\[dependencies]

dax\_agent\_orchestrator = "0.2"

If you are developing locally and the orchestrator is in a sibling folder:



toml

\[dependencies]

dax\_agent\_orchestrator = { path = "../dax\_agent\_orchestrator" }

Step 2 — Enable features you need



Common features:



with-async — enables Tokio-backed parallel runner and async helpers.



with-serde — enables optional JSON payload support in Task.



Example enabling both:



toml

\[dependencies.dax\_agent\_orchestrator]

version = "0.2"

features = \["with-async", "with-serde"]

Feature flags and runtime requirements

with-async



Use this when your host uses Tokio or you want the Tokio-backed run\_subagents\_parallel.



Ensure your binary depends on Tokio (e.g., tokio = { version = "1", features = \["rt-multi-thread", "macros"] }).



If you do not enable with-async, the library falls back to a thread-based parallel runner.



with-serde



Enables Task structured payload support.



Add serde to your dependency list if you plan to use JSON payloads.



Example Cargo.toml snippet for a host binary:



toml

\[dependencies]

dax\_agent\_orchestrator = { version = "0.2", features = \["with-async", "with-serde"] }

tokio = { version = "1", features = \["rt-multi-thread", "macros"] }

serde = { version = "1", features = \["derive"] }

Implementing the required traits

You must implement AgentState, provide concrete Delta types that implement DeltaState, and implement an executor that runs subagents.



1\. Implement AgentState



rust

use dax\_agent\_orchestrator::traits::{AgentState, DeltaState};



\#\[derive(Clone, Debug)]

struct MyState { /\* your fields \*/ }



impl AgentState for MyState {

&#x20;   fn apply\_delta(\&mut self, delta: \&dyn DeltaState) {

&#x20;       if let Some(d) = delta.as\_any().downcast\_ref::<MyDelta>() {

&#x20;           // merge logic

&#x20;           self.counter += d.delta;

&#x20;       } else {

&#x20;           // handle unknown delta types

&#x20;       }

&#x20;   }

}

2\. Define concrete delta types



rust

use std::fmt::Debug;



\#\[derive(Debug)]

struct MyDelta { delta: i64 }



// Blanket impls in the library make this a DeltaState automatically

// because it implements Send + Debug + Any + 'static.

3\. Implement an executor



Synchronous executor:



rust

use dax\_agent\_orchestrator::traits::{AgentExecutor, Task};



struct MyExecutor;



impl AgentExecutor<MyState> for MyExecutor {

&#x20;   fn run(\&self, state: MyState, task: Task) -> Box<dyn dax\_agent\_orchestrator::traits::DeltaState + Send> {

&#x20;       // perform work and return a boxed delta

&#x20;       Box::new(MyDelta { delta: 1 })

&#x20;   }

}

Async executor (optional):



rust

use dax\_agent\_orchestrator::traits::AgentExecutorAsync;

use std::pin::Pin;

use std::future::Future;



struct MyAsyncExecutor;



impl AgentExecutorAsync<MyState> for MyAsyncExecutor {

&#x20;   type Fut = Pin<Box<dyn Future<Output = Box<dyn dax\_agent\_orchestrator::traits::DeltaState + Send>> + Send>>;



&#x20;   fn run\_async(\&self, state: MyState, task: Task) -> Self::Fut {

&#x20;       Box::pin(async move {

&#x20;           // async work

&#x20;           Box::new(MyDelta { delta: 1 })

&#x20;       })

&#x20;   }

}

Running the split → execute → collapse flow

1\. Create SubAgentSpecs with split



rust

use dax\_agent\_orchestrator::{split, SplitStrategy, SubAgentSpec, Task};



let master: MyState = /\* ... \*/;

let tasks = vec!\[ Task::new("t1", "payload1"), Task::new("t2", "payload2") ];



let specs: Vec<SubAgentSpec<MyState>> =

&#x20;   split(\&master, SplitStrategy::SemanticRouting, tasks, |s, \_i| s.clone());

2\. Run subagents synchronously



rust

use dax\_agent\_orchestrator::run\_subagents\_local;



let exec = MyExecutor;

let results = run\_subagents\_local(specs, \&exec);

// results: Vec<SubAgentResult> with id, delta, metadata

3\. Run subagents in parallel (Tokio)



Enable with-async and run:



rust

use dax\_agent\_orchestrator::run\_subagents\_parallel;

use std::sync::Arc;



let exec\_arc = Arc::new(MyExecutor);

let results = run\_subagents\_parallel(specs, exec\_arc).await;

4\. Collapse deltas back into master



Default sequential collapse:



rust

use dax\_agent\_orchestrator::{collapse\_from\_id\_pairs, CollapseStrategy};



let id\_and\_deltas: Vec<(String, Box<dyn dax\_agent\_orchestrator::traits::DeltaState + Send>)> =

&#x20;   results.into\_iter().map(|r| (r.id, r.delta)).collect();



let new\_master = collapse\_from\_id\_pairs(master, id\_and\_deltas, CollapseStrategy::Sequential);

5\. Custom merge logic



If you need weighting or provenance-aware merging, use collapse\_with and provide a merge\_fn:



rust

use dax\_agent\_orchestrator::collapse\_with;



let merged = collapse\_with(master, deltas\_with\_ids, |master, delta, id\_opt| {

&#x20;   // custom merge using id\_opt and delta downcast

});

Testing, CI, and troubleshooting

Run tests locally



Unit tests:



bash

cargo test

Run async tests (Tokio-backed):



bash

cargo test --features "with-async"

Run ignored tests:



bash

cargo test -- --ignored

Run doctests:



bash

cargo test --doc

CI recommendations



Matrix builds:



cargo test (default features)



cargo test --features "with-async"



cargo test --features "with-serde"



Linting:



bash

cargo clippy --all-targets --all-features -- -D warnings

Format check:



bash

cargo fmt -- --check

Common troubleshooting



Downcast returns None: ensure the concrete delta type is the same type used by the executor and that it implements Send + Debug + Any + 'static. Use delta.as\_any().downcast\_ref::<YourDelta>().



Feature-gated tests skipped: enable the required features on the cargo test command line.



Tokio runtime errors: ensure your binary includes Tokio and that with-async is enabled in Cargo.toml.



Unused import warnings in examples: conditionally import run\_subagents\_parallel and Arc with #\[cfg(feature = "with-async")].



Panics in tests: inspect unwrap() calls in tests; prefer graceful error handling or assert messages.

