\# dax\_agent\_orchestrator



A small, drop-in DAX-style orchestrator crate that lets a host agent split its state into subagents, run them, and collapse deltas back.



\## Key design goals

\- \*\*Opaque host types\*\*: the crate never forces a memory format.

\- \*\*Minimal API\*\*: `split`, `run\_subagents\_\*`, `collapse`.

\- \*\*Safe downcasting\*\*: `DeltaState` exposes `as\_any` for host-specific downcasts.

\- \*\*Sync and async runners\*\*: example shows both.



\## Example

See `examples/host\_agent.rs` for a complete host integration.



\## Integration

1\. Implement `AgentState` for your memory type.

2\. Produce concrete delta types (they automatically implement `DeltaState`).

3\. Implement `AgentExecutor` to run a subagent.

4\. Use `split`, run subagents, then `collapse`.



\## Notes

\- Replace the `extract\_slice` closure with semantic slicing logic (embeddings, key filtering).

\- Implement richer collapse strategies in your `AgentState::apply\_delta` or extend `dax::collapse`.



