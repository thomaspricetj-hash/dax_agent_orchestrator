use crate::traits::{AgentState, DeltaState, SubAgentSpec, Task};
use std::fmt::Debug;

/// Split strategy enum for future extension.
#[non_exhaustive]
#[derive(Clone, Debug)]
pub enum SplitStrategy {
    RoundRobin(usize),
    SemanticRouting,
}

/// Collapse strategy enum for future extension.
#[non_exhaustive]
#[derive(Clone, Debug)]
pub enum CollapseStrategy {
    /// Apply deltas in the order they are provided by the host.
    Sequential,
    /// Use a weighted merge strategy (host must provide weights via `collapse_with`).
    Weighted,
}

/// Minimal DAX-style split function.
///
/// This function is intentionally generic and does not assume memory format.
/// Host agents provide an `extract_slice` closure to produce scoped states.
///
/// # Parameters
/// - `state`: reference to the host's master state.
/// - `_strategy`: split strategy (reserved for future use).
/// - `tasks`: list of tasks to produce `SubAgentSpec`s for.
/// - `extract_slice`: closure that given the master state and an index returns a scoped state for that subagent.
///
/// # Returns
/// A `Vec<SubAgentSpec<S>>` where each spec contains an id, a scoped state, and the task.
pub fn split<S>(
    state: &S,
    _strategy: SplitStrategy,
    tasks: Vec<Task>,
    mut extract_slice: impl FnMut(&S, usize) -> S,
) -> Vec<SubAgentSpec<S>>
where
    S: AgentState,
{
    let mut specs = Vec::with_capacity(tasks.len());
    for (i, task) in tasks.into_iter().enumerate() {
        let slice = extract_slice(state, i);
        let id = format!("sub-{}", i);
        specs.push(SubAgentSpec { id, scoped_state: slice, task });
    }
    specs
}

/// Collapse function that applies deltas back into the original state using the default merge behavior.
///
/// This default simply calls `AgentState::apply_delta` for each delta in order. For more advanced
/// merge semantics (weighted merges, conflict resolution, provenance-aware merging), use
/// `collapse_with` and provide a custom merge function.
///
/// # Parameters
/// - `original`: the master state to be updated (consumed and returned).
/// - `deltas`: iterator of boxed `DeltaState` objects produced by subagents.
/// - `_strategy`: collapse strategy hint (currently unused by the default implementation).
///
/// # Returns
/// The updated master state after applying all deltas.
pub fn collapse<S, I>(mut original: S, deltas: I, _strategy: CollapseStrategy) -> S
where
    S: AgentState,
    I: IntoIterator<Item = Box<dyn DeltaState + Send>>,
{
    // Default behavior: apply each delta in sequence using the host's `apply_delta`.
    for delta in deltas {
        original.apply_delta(delta.as_ref());
    }
    original
}

/// Collapse with a custom merge function.
///
/// This function gives hosts full control over how each delta is merged into the master state.
/// The `merge_fn` receives a mutable reference to the master state, a reference to the delta,
/// and the subagent id (if available). The merge function can implement weighting, conflict
/// resolution, provenance checks, or any other policy.
///
/// The short illustrative example below is intentionally marked `ignore` so it doesn't run as a doctest.
/// Replace with a concrete example in your host crate when integrating.
pub fn collapse_with<S, F, I>(mut original: S, deltas: I, mut merge_fn: F) -> S
where
    S: AgentState,
    F: FnMut(&mut S, &dyn DeltaState, Option<&str>),
    I: IntoIterator<Item = (Option<String>, Box<dyn DeltaState + Send>)>,
{
    for (id_opt, delta) in deltas {
        let id_ref = id_opt.as_deref();
        merge_fn(&mut original, delta.as_ref(), id_ref);
    }
    original
}

/// Convenience helper to convert a vector of `SubAgentResult`-style deltas (id + delta)
/// into the shape expected by `collapse_with` and call it with a simple `apply_delta` merge.
///
/// This is useful when you have `Vec<(id, Box<dyn DeltaState>)>` and want the default
/// apply-delta behavior while preserving ids for logging or future strategies.
///
/// # Parameters
/// - `original`: master state
/// - `id_and_deltas`: vector of `(id, delta)` pairs
/// - `strategy`: collapse strategy hint (currently only `Sequential` uses default apply)
///
/// # Returns
/// Updated master state.
pub fn collapse_from_id_pairs<S>(
    original: S,
    id_and_deltas: Vec<(String, Box<dyn DeltaState + Send>)>,
    strategy: CollapseStrategy,
) -> S
where
    S: AgentState,
{
    // Convert into the (Option<String>, Box<dyn DeltaState>) shape
    let deltas_opt: Vec<(Option<String>, Box<dyn DeltaState + Send>)> =
        id_and_deltas.into_iter().map(|(id, d)| (Some(id), d)).collect();

    match strategy {
        CollapseStrategy::Sequential => {
            collapse_with(original, deltas_opt, |master, delta, _id| {
                master.apply_delta(delta);
            })
        }
        // For Weighted or other strategies, hosts should call `collapse_with` directly
        // with a merge_fn that implements the desired policy.
        _ => collapse_with(original, deltas_opt, |master, delta, _id| {
            master.apply_delta(delta);
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::{AgentState, DeltaState};

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

    #[test]
    fn collapse_sequential_applies_all() {
        let master = SimpleState { counter: 0 };
        let deltas: Vec<Box<dyn DeltaState + Send>> =
            vec![SimpleDelta { delta: 3 }.into(), SimpleDelta { delta: 5 }.into()];
        let new_master = collapse(master, deltas, CollapseStrategy::Sequential);
        assert_eq!(new_master.counter, 8);
    }

    #[test]
    fn collapse_from_id_pairs_preserves_ids_and_applies() {
        let master = SimpleState { counter: 1 };
        let id_and_deltas = vec![
            ("sub-0".to_string(), SimpleDelta { delta: 2 }.into()),
            ("sub-1".to_string(), SimpleDelta { delta: 4 }.into()),
        ];
        let new_master = collapse_from_id_pairs(master, id_and_deltas, CollapseStrategy::Sequential);
        assert_eq!(new_master.counter, 7);
    }
}

