DAX Agent Orchestrator — Whitepaper

Executive summary

This whitepaper describes the DAX Agent Orchestrator, a lightweight Rust framework for splitting a host agent’s state into scoped subagents, executing those subagents (synchronously or asynchronously), and collapsing their deltas back into the master state. The design emphasizes type-safe extensibility, host-controlled merge semantics, and runtime-agnostic execution so hosts can integrate with minimal friction while retaining full control over state representation and merging policies.



Background and goals

Problem space: Complex agents often need to decompose reasoning or work into smaller, concurrent tasks that operate on scoped views of a larger state. Coordinating those subagents, collecting their outputs, and merging results back into a single authoritative state is nontrivial and host-specific.



Primary goals



Host ownership of state: Hosts define AgentState and concrete DeltaState types so merging logic remains in host code.



Safe downcasting: Provide ergonomic, object-safe downcasting for deltas so hosts can inspect and apply them.



Flexible execution: Support synchronous and asynchronous execution models, with a simple thread fallback.



Pluggable merge strategies: Offer default sequential application and a collapse\_with hook for custom merge policies (weighted merges, provenance-aware resolution).



Minimal runtime assumptions: Avoid forcing a particular async runtime or serialization format.



Architecture overview

Core concepts

AgentState: Host-defined, Clone + Send + Debug + 'static, with an apply\_delta(\&mut self, delta: \&dyn DeltaState) method. Hosts implement how deltas mutate the master state.



DeltaState: Trait object representing changes produced by subagents. It exposes as\_any() for safe downcasting. Blanket impls let any Send + Debug + Any + 'static type be a DeltaState.



Task: Lightweight descriptor passed to subagents. Contains name and payload (string) and optionally structured JSON when with-serde is enabled.



SubAgentSpec: Bundles an id, a scoped copy/view of the host state, and a Task.



AgentExecutor / AgentExecutorAsync: Host-implemented executors for running subagents synchronously or asynchronously. Async trait uses an associated Fut type; a convenience BoxDeltaFuture alias is provided for boxed futures.



Data flow

Split: split(state, strategy, tasks, extract\_slice) produces Vec<SubAgentSpec<S>>. The host supplies extract\_slice to create scoped states.



Execute: Executors run each SubAgentSpec and return Box<dyn DeltaState + Send>.



Collapse: collapse or collapse\_with merges deltas back into the master state. Hosts can use collapse\_from\_id\_pairs to preserve subagent ids for logging or provenance-aware merges.



API and semantics

Traits and ergonomics

AgentState



Must be Clone so split can create scoped copies by default.



apply\_delta is the canonical merge hook for default sequential merging.



DeltaState



Exposes as\_any() for downcasting.



An inherent helper downcast\_ref<T>() on dyn DeltaState is provided to simplify host code.



IntoBoxedDelta



Convenience trait so concrete delta values can be converted to Box<dyn DeltaState + Send> with into\_boxed().



Task and SubAgentSpec

Task::new(name, payload) is the primary constructor.



SubAgentSpec carries id, scoped\_state, and task. IDs are host-chosen strings (e.g., sub-0, reasoning::1).



Split and collapse functions

split: Generic, host-supplied extract\_slice controls how the master state is partitioned.



collapse: Default sequential application using AgentState::apply\_delta.



collapse\_with: Accepts (Option<String>, Box<dyn DeltaState + Send>) items and a merge\_fn(\&mut S, \&dyn DeltaState, Option<\&str>) for custom merging.



collapse\_from\_id\_pairs: Convenience wrapper converting (String, Box<dyn DeltaState>) into the shape expected by collapse\_with.



Execution models and concurrency

Synchronous execution

AgentExecutor::run(\&self, state: S, task: Task) -> Box<dyn DeltaState + Send> is the synchronous contract.



run\_subagents\_local is a simple helper that iterates specs and calls the executor sequentially, returning Vec<SubAgentResult> with optional metadata.



Asynchronous execution

AgentExecutorAsync provides type Fut and run\_async.



The library offers run\_subagents\_parallel:



Tokio-backed (when with-async feature enabled): uses tokio::task::spawn\_blocking to run blocking executors concurrently and collects results in the same order as specs.



Thread fallback (no with-async): uses std::thread::spawn to parallelize execution.



Ordering guarantee: The parallel runner returns results in the same order as the input specs by collecting join handles in order and awaiting them in sequence.



Metadata and provenance

SubAgentResult includes id, delta, and optional metadata: Option<HashMap<String,String>>. Hosts can populate metadata (executor id, latency, confidence) either in the helper or via richer executor implementations.



Concurrency, safety, and object-safety

Object-safety: DeltaState remains object-safe because downcast helpers are implemented as inherent methods on dyn DeltaState rather than generic trait methods.



Send bounds: AgentState and DeltaState are Send to allow cross-thread execution.



Clone semantics: split clones the master state by default; hosts that cannot cheaply clone should provide extract\_slice that returns lightweight views or references (if they implement appropriate lifetimes and safety).



Panic handling: Parallel runners skip panicked tasks by default; hosts can adapt to surface errors or retry.



Downcasting: Hosts should always downcast deltas using as\_any().downcast\_ref::<ConcreteDelta>() or the provided downcast\_ref helper to avoid unwrap() panics.



Testing, examples, and integration

Unit tests

The repository includes unit tests that:



Validate downcasting and apply\_delta.



Validate run\_subagents\_local and run\_subagents\_parallel behaviors.



Validate collapse and collapse\_from\_id\_pairs.



Doctests: Illustrative doc examples are marked ignore to avoid compilation failures; hosts should replace them with concrete examples when integrating.



Example host

A small example (examples/host\_agent.rs) demonstrates:



Implementing AgentState and DeltaState.



Implementing AgentExecutor.



Running both synchronous and async flows (async path gated by with-async feature).



Collapsing results back into the master state.



Recommended test matrix

Run unit tests with and without with-async.



Run doctests after replacing ignore with concrete examples.



Add integration tests that simulate real host state shapes and complex merge policies.



Performance and benchmarks

Microbenchmarks



Measure split cost for host state cloning or slicing.



Measure executor latency per subagent and aggregate throughput.



Measure collapse cost for sequential vs. custom merge functions.



Parallelism tradeoffs



Tokio-backed spawn\_blocking is suitable for blocking executors; prefer AgentExecutorAsync for fully async subagents.



Thread fallback is simple but may be heavier for many small tasks; prefer runtime-aware async execution for high concurrency.



Optimization tips



Use lightweight scoped states (views or references) when cloning is expensive.



Batch small tasks to reduce scheduling overhead.



Provide provenance metadata to enable selective reapplication or conflict resolution without full recomputation.



Integration and migration guidance

Adopting in an existing host



Implement AgentState for your master state and define concrete delta types.



Implement AgentExecutor (and optionally AgentExecutorAsync) to run subagents.



Use split with a host extract\_slice to produce SubAgentSpecs.



Run subagents via run\_subagents\_local or run\_subagents\_parallel.



Merge results using collapse or collapse\_with depending on your merge policy.



Serialization and persistence



If you need to persist deltas, ensure concrete delta types implement Serialize/Deserialize under a with-serde feature and store type tags for safe deserialization.



Cross-language integration



Expose a thin RPC layer where subagents run in other languages; serialize Task and Delta payloads as JSON/protobuf and implement host-side deserialization into concrete delta types.



Roadmap and extensions

Provenance-aware merging: Add first-class provenance metadata and merge strategies that use confidence scores or timestamps.



Conflict resolution policies: Provide built-in strategies (last-writer-wins, weighted average, CRDT-based merges).



Pluggable schedulers: Add scheduling strategies (priority, rate-limited, resource-aware).



Observability: Add hooks for tracing, metrics, and structured logging for each subagent run.



Typed delta registry: Optional registry to map type tags to concrete delta deserializers for persisted deltas.



Language bindings: Provide adapters for other languages or a gRPC interface for remote subagents.



Appendix

Key types



AgentState, DeltaState, IntoBoxedDelta, Task, SubAgentSpec, AgentExecutor, AgentExecutorAsync, BoxDeltaFuture.



Helper functions



split, collapse, collapse\_with, collapse\_from\_id\_pairs, run\_subagents\_local, run\_subagents\_parallel.



Testing notes



Use cargo test --features "with-async" to exercise async paths.



Replace ignore doctest blocks with concrete examples before running cargo test --doc.

