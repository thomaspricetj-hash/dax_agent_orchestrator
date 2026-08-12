DAX Agent Orchestrator
A lightweight, drop‑in Rust orchestrator for splitting a host agent’s state into scoped subagents, executing them (synchronously or asynchronously), and collapsing their deltas back into a unified master state. Inspired by DAX‑style decomposition, but intentionally minimal, host‑controlled, and runtime‑agnostic.

The orchestrator is designed for cognitive agents, simulation engines, distributed reasoning systems, and any workload that benefits from structured task decomposition and deterministic state merging.

Features
✔ Host‑owned state
You define AgentState and your concrete delta types. The orchestrator never imposes a memory format or serialization model.

✔ Minimal API surface
Only the essentials:

split

run_subagents_local

run_subagents_parallel

collapse

collapse_with

collapse_from_id_pairs

✔ Safe, ergonomic downcasting
DeltaState exposes as_any() and downcast_ref<T>() so hosts can inspect concrete delta types without unsafe code.

✔ Sync + async execution
Pure synchronous execution

Tokio‑backed parallel execution (optional)

Thread‑based fallback when async is disabled

✔ Pluggable merge semantics
Use the default sequential merge or provide your own merge logic via:

AgentState::apply_delta

collapse_with

provenance‑aware merging using collapse_from_id_pairs

✔ No runtime assumptions
No required async runtime, no serialization requirements, no global registries.

Quick Example
A complete example is available in:

Code
examples/host_agent.rs
It demonstrates:

Implementing AgentState

Defining concrete delta types

Implementing AgentExecutor

Running sync and async flows

Collapsing results back into the master state

How to Integrate
1. Implement AgentState
Your master state must implement:

rust
fn apply_delta(&mut self, delta: &dyn DeltaState)
This is your merge hook.

2. Define concrete delta types
Any Send + Debug + Any + 'static type automatically implements DeltaState.

3. Implement an executor
Define how a subagent runs:

rust
impl AgentExecutor<MyState> for MyExecutor {
    fn run(&self, state: MyState, task: Task) -> Box<dyn DeltaState + Send> {
        // produce a delta
    }
}
4. Split → Run → Collapse
A typical flow:

rust
let specs = split(&master_state, strategy, tasks, |s, _| s.clone());
let results = run_subagents_local(specs, &executor);
let new_master = collapse(master_state, results.into_iter().map(|r| r.delta), CollapseStrategy::Sequential);
Advanced Usage
Semantic slicing
Replace the extract_slice closure with logic that produces scoped views:

embedding‑based routing

key‑filtered state slices

partial memory views

role‑specific sub‑states

Custom merge strategies
Extend merging via:

custom logic inside apply_delta

weighted merges

provenance‑aware merges

conflict resolution policies

Async execution
Enable the with-async feature:

bash
cargo run --example host_agent --features "with-async"
This activates Tokio‑backed parallel execution.

API Overview
Split
rust
split(state, strategy, tasks, extract_slice)
Execute
rust
run_subagents_local(specs, executor)
run_subagents_parallel(specs, Arc::new(executor)).await
Collapse
rust
collapse(master, deltas, CollapseStrategy::Sequential)
collapse_with(master, id_deltas, merge_fn)
collapse_from_id_pairs(master, id_delta_pairs, strategy)
Design Notes
The orchestrator is intentionally small and host‑controlled.

All deltas are object‑safe and downcastable.

Parallel execution preserves input ordering.

Panic isolation ensures failed subagents do not break the pipeline.

No global state, no macros, no magic.

Roadmap
Planned enhancements:

provenance‑aware merging

richer collapse strategies

priority‑based scheduling

typed delta registries

observability hooks (tracing, metrics)

language bindings (Python, JS, gRPC)



