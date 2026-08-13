//! MAX‑TIER AGENT TREE
//! Hierarchical agent recursion, depth/cost guards, provenance.

use std::fmt::Debug;
use std::marker::PhantomData;

use crate::core::traits::{AgentState, FractalAgent, CostPredictor, Task};

/// Minimal local definitions for AgentNode and AgentTree.
///
/// These are intentionally lightweight, self-contained types so this module
/// compiles even if the project does not provide a shared `AgentNode`/`AgentTree`
/// type in `core::traits`. If you already have canonical definitions elsewhere,
/// replace these with `use` imports pointing to the canonical module.
#[derive(Clone, Debug)]
pub struct AgentNode {
    pub id: String,
    pub depth: usize,
    pub children: Vec<AgentNode>,
}

impl AgentNode {
    pub fn new(id: &str, depth: usize) -> Self {
        Self {
            id: id.to_string(),
            depth,
            children: Vec::new(),
        }
    }

    pub fn add_child(&mut self, child: AgentNode) {
        self.children.push(child);
    }
}

#[derive(Clone, Debug)]
pub struct AgentTree {
    pub root: AgentNode,
    pub max_depth: usize,
    pub max_cost: usize,
}

impl AgentTree {
    pub fn new(root_id: &str) -> Self {
        Self {
            root: AgentNode::new(root_id, 0),
            max_depth: 8,
            max_cost: 10_000,
        }
    }
}

/// Runtime node state in the agent tree.
#[derive(Clone, Debug)]
pub struct AgentTreeNodeState<S: AgentState> {
    pub node: AgentNode,
    pub state: S,
}

impl<S: AgentState> AgentTreeNodeState<S> {
    pub fn new(id: &str, depth: usize, state: S) -> Self {
        Self {
            node: AgentNode::new(id, depth),
            state,
        }
    }
}

/// Execution context for a recursive agent tree run.
#[derive(Clone, Debug)]
pub struct AgentTreeContext<S: AgentState> {
    pub tree: AgentTree,
    pub nodes: Vec<AgentTreeNodeState<S>>,
}

impl<S: AgentState> AgentTreeContext<S> {
    /// Construct a new context for a given root id and root state.
    ///
    /// Note: we accept `root_id` by value (`String`) to avoid borrow/lifetime
    /// friction when callers produce an owned id. Internally we pass `&str`
    /// to the `AgentTree` and `AgentNode` constructors.
    pub fn new(root_id: String, root_state: S) -> Self {
        let mut tree = AgentTree::new(root_id.as_str());
        let root_node = AgentNode::new(root_id.as_str(), 0);
        tree.root = root_node;

        let root = AgentTreeNodeState::new(root_id.as_str(), 0, root_state);

        Self {
            tree,
            nodes: vec![root],
        }
    }

    pub fn max_depth(&self) -> usize {
        self.tree.max_depth
    }

    pub fn max_cost(&self) -> usize {
        self.tree.max_cost
    }

    pub fn add_child(
        &mut self,
        parent_idx: usize,
        child_id: &str,
        child_state: S,
    ) -> usize {
        let parent_depth = self.nodes[parent_idx].node.depth;
        let child_depth = parent_depth + 1;

        let child_node = AgentNode::new(child_id, child_depth);
        self.tree.root.add_child(child_node.clone());

        let child = AgentTreeNodeState {
            node: child_node,
            state: child_state,
        };

        self.nodes.push(child);
        self.nodes.len() - 1
    }
}

/// Recursive executor over a fractal agent.
///
/// `S` must be `Clone` because the executor clones state for child nodes.
/// If you prefer a different ownership model, change the executor to pass
/// references or move ownership into subtasks.
pub struct AgentTreeExecutor<S: AgentState + Clone, A: FractalAgent<S> + CostPredictor<S>> {
    pub agent: A,
    // `S` appears only in trait bounds on `A` and method signatures; include a
    // PhantomData to avoid "type parameter is never used" warnings.
    _state_marker: PhantomData<S>,
}

impl<S: AgentState + Clone, A: FractalAgent<S> + CostPredictor<S>> AgentTreeExecutor<S, A> {
    pub fn new(agent: A) -> Self {
        Self {
            agent,
            _state_marker: PhantomData,
        }
    }

    pub fn run(&self, root_state: S, root_task: Task) -> AgentTreeContext<S> {
        let mut ctx = AgentTreeContext::new(self.agent_root_id(), root_state);
        self.recurse(&mut ctx, 0, root_task);
        ctx
    }

    fn agent_root_id(&self) -> String {
        "root".to_string()
    }

    fn recurse(&self, ctx: &mut AgentTreeContext<S>, node_idx: usize, task: Task) {
        let depth = ctx.nodes[node_idx].node.depth;
        if depth >= self.agent.max_fractal_depth() {
            return;
        }

        let state = ctx.nodes[node_idx].state.clone();
        let split = self.agent.split_task(&state, &task, depth);
        if split.is_none() {
            return;
        }

        let split = split.unwrap();
        let cost = self.agent.predict_many(&state, &split.sub_tasks);
        if cost > self.agent.max_fractal_cost() {
            return;
        }

        for sub in split.sub_tasks {
            let child_state = state.clone();
            let child_idx = ctx.add_child(node_idx, &sub.name, child_state);
            self.recurse(ctx, child_idx, sub);
        }
    }
}


