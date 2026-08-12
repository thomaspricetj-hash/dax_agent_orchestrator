============================================================
DAX Agent Orchestrator – Max‑Tier Install & Integration Guide
============================================================

Overview
A complete guide for installing and integrating the Max‑Tier DAX Agent Orchestrator into a Rust host agent. Covers dependency setup, feature flags, required trait implementations, tier configuration, ledger and telemetry usage, sync/async orchestration, testing, CI, and troubleshooting.

This guide reflects the full Max‑Tier upgrade including:

DaxLedger (provenance tracking)

DaxTelemetry (heatmaps, influence edges, collapse order)

Tier‑aware split/collapse strategies

Weighted collapse

Fractal expansion

Updated return signatures

Updated exports in lib.rs

Updated example host agent

Add dependency

If using crates.io:
[dependencies]
dax_agent_orchestrator = "0.2"

If developing locally:
[dependencies]
dax_agent_orchestrator = { path = "../dax_agent_orchestrator" }

Enable features

with-async
Enables Tokio-backed parallel execution.

with-serde
Enables JSON payload support in Task.

Example:
[dependencies.dax_agent_orchestrator]
version = "0.2"
features = ["with-async", "with-serde"]

Tokio requirement when using async:
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }

Serde requirement when using JSON payloads:
serde = { version = "1", features = ["derive"] }

Max‑Tier API Exports

Your lib.rs must export:

SplitStrategy
CollapseStrategy
DaxTier
DaxTelemetry
DaxLedger
dax_run_sync
dax_run_async
dax_split
dax_execute_sync
dax_execute_async
dax_collapse

These are required for Max‑Tier orchestration.

Implement required traits

AgentState
Your state must implement:
apply_delta(&mut self, delta: &dyn DeltaState)

DeltaState
Any Send + Debug + Any + 'static type automatically implements DeltaState.

Example delta:
struct MyDelta { delta: i64 }

Executor (sync):
run(&self, state, task) -> Box<dyn DeltaState + Send>

Executor (async):
run_async(&self, state, task) -> Future returning Box<dyn DeltaState + Send>

Max‑Tier orchestration flow

The Max‑Tier orchestrator returns:

Sync:
(new_master, telemetry, ledger)

Async:
(new_master, telemetry, ledger)

Telemetry includes:
agent_heat
delta_heat
influence_edges
collapse_order

Ledger includes:
agent_id
task_name
delta_type
delta_value
depth
cost
collapse_position
influenced
timestamp

Split → Execute → Collapse (Max‑Tier)

Split:
specs = dax_split(agent, &master, split_strategy, tasks, |s, i| s.clone())

Execute (sync):
(results, telemetry, ledger) = dax_execute_sync(specs, executor)

Execute (async):
(results, telemetry, ledger) = dax_execute_async(specs, executor).await

Collapse:
new_master = dax_collapse(master, results, collapse_strategy, &mut telemetry, &mut ledger)

Tier configuration

Tier1Basic
Sequential collapse

Tier2Weighted
Weighted collapse

Tier3Adaptive
Semantic routing + host collapse

Tier4FractalBoost
Semantic routing + weighted collapse + fractal expansion

Tier5Cognitive
Full Max‑Tier mode:

Semantic routing

Weighted collapse

Fractal expansion

Telemetry

Ledger

Influence tracking

Collapse position tracking

Example Max‑Tier host usage

(new_master, telemetry, ledger) = dax_run_sync(
&agent,
&executor,
master,
tasks,
SplitStrategy::SemanticRouting,
CollapseStrategy::Weighted,
DaxTier::Tier5Cognitive,
|s, _| s.clone(),
);

Print telemetry:
telemetry.agent_heat
telemetry.delta_heat
telemetry.influence_edges
telemetry.collapse_order

Print ledger:
ledger.entries

Testing and CI

Local tests:
cargo test

Async tests:
cargo test --features "with-async"

Serde tests:
cargo test --features "with-serde"

CI matrix:
cargo test
cargo test --features "with-async"
cargo test --features "with-serde"

Lint:
cargo clippy --all-targets --all-features -- -D warnings

Format:
cargo fmt -- --check

Troubleshooting

Downcast returns None
Delta type mismatch. Ensure executor returns the correct concrete delta.

Tokio runtime errors
Binary must include Tokio and with-async must be enabled.

Missing ledger or telemetry
Ensure lib.rs exports DaxLedger and DaxTelemetry.

Unused imports
Use #[cfg(feature = "with-async")] for async-only imports.

Collapse order incorrect
Ensure collapse_strategy matches tier configuration.

Influence edges empty
Weighted collapse or Tier5Cognitive must be enabled.

Recommended host architecture

Use lightweight scoped states.
Use semantic routing for meaningful splits.
Use weighted collapse for confidence-based merging.
Use ledger for provenance and debugging.
Use telemetry for performance tuning.
Use fractal expansion for recursive reasoning.

Roadmap

Planned enhancements:
Provenance-aware merging
Priority-based scheduling
Typed delta registries
Observability hooks
Language bindings (Python, JS, gRPC)
Predictive Tier‑6 routing