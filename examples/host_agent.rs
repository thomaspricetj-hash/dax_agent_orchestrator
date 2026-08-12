// examples/host_agent.rs
// Run with:
//   cargo run --example host_agent --features "with-async"
// or without async:
//   cargo run --example host_agent

use dax_agent_orchestrator as orchestrator;
use orchestrator::{
    Agent, AgentState, DeltaState, MicroAgent, FractalAgent, FractalConfig,
    Task, SplitStrategy, CollapseStrategy,
};

#[cfg(not(feature = "with-async"))]
use orchestrator::dax_run_sync;

#[cfg(feature = "with-async")]
use orchestrator::dax_run_async;

use std::sync::Arc;
use std::fmt::Debug;

// ============================================================================
// STATE + DELTA
// ============================================================================

#[derive(Clone, Debug)]
struct SimpleState {
    pub counter: i64,
}

impl AgentState for SimpleState {
    fn apply_delta(&mut self, delta: &dyn DeltaState) {
        if let Some(d) = delta.downcast_ref::<SimpleDelta>() {
            self.counter += d.delta;
        } else {
            eprintln!("apply_delta: unknown delta type {:?}", delta);
        }
    }
}

#[derive(Debug)]
struct SimpleDelta {
    delta: i64,
}

// ============================================================================
// MICRO‑AGENT IMPLEMENTATION
// ============================================================================

#[derive(Debug)]
struct IncrementAgent;

impl MicroAgent<SimpleState> for IncrementAgent {
    fn accepts(&self, task: &Task) -> bool {
        task.name.starts_with("inc")
    }

    fn execute(&self, _state: SimpleState, task: Task) -> Box<dyn DeltaState + Send> {
        let inc = task.payload.parse::<i64>().unwrap_or(1);
        println!("[micro] incrementing by {}", inc);
        Box::new(SimpleDelta { delta: inc })
    }
}

// ============================================================================
// FRACTAL AGENT IMPLEMENTATION
// ============================================================================

#[derive(Debug)]
struct RecursiveAgent;

impl FractalAgent<SimpleState> for RecursiveAgent {
    fn split(&self, state: SimpleState, depth: usize)
        -> Vec<orchestrator::SubAgentSpec<SimpleState>>
    {
        let next = depth as i64;
        let task = Task::new(format!("inc-depth-{}", depth), next.to_string());

        vec![orchestrator::SubAgentSpec {
            id: format!("fractal-{}", depth),
            scoped_state: state.clone(),
            task,
        }]
    }

    fn estimate_cost(&self, _state: &SimpleState) -> usize {
        1
    }

    fn config(&self) -> FractalConfig {
        FractalConfig {
            max_depth: 3,
            max_cost: 128,
        }
    }
}

// ============================================================================
// UNIFIED AGENT IMPLEMENTATION
// ============================================================================

#[derive(Debug)]
struct HostAgent {
    micro: Arc<IncrementAgent>,
    fractal: Arc<RecursiveAgent>,
}

impl Agent<SimpleState> for HostAgent {
    fn as_micro(&self) -> Option<Arc<dyn MicroAgent<SimpleState>>> {
        Some(self.micro.clone())
    }

    fn as_fractal(&self) -> Option<Arc<dyn FractalAgent<SimpleState>>> {
        Some(self.fractal.clone())
    }
}

// ============================================================================
// EXECUTOR
// ============================================================================

#[derive(Debug)]
struct HostExecutor;

impl orchestrator::AgentExecutor<SimpleState> for HostExecutor {
    fn run(&self, _state: SimpleState, task: Task) -> Box<dyn DeltaState + Send> {
        println!("[executor] running {} with payload {:?}", task.name, task.payload);

        if task.name.starts_with("inc") {
            let inc = task.payload.parse::<i64>().unwrap_or(1);
            return Box::new(SimpleDelta { delta: inc });
        }

        Box::new(SimpleDelta { delta: 1 })
    }
}

// ============================================================================
// MAIN (SYNC + ASYNC)
// ============================================================================

#[cfg(feature = "with-async")]
#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let agent = HostAgent {
        micro: Arc::new(IncrementAgent),
        fractal: Arc::new(RecursiveAgent),
    };

    let executor = Arc::new(HostExecutor);

    let master = SimpleState { counter: 0 };
    let tasks = vec![
        Task::new("inc-A", "3"),
        Task::new("inc-B", "5"),
        Task::new("root", "1"),
    ];

    let new_master = dax_run_async(
        &agent,
        executor,
        master,
        tasks,
        SplitStrategy::SemanticRouting,
        CollapseStrategy::Sequential,
        |s, _| s.clone(),
    ).await;

    println!("Final master state (async): {:?}", new_master);
}

#[cfg(not(feature = "with-async"))]
fn main() {
    let agent = HostAgent {
        micro: Arc::new(IncrementAgent),
        fractal: Arc::new(RecursiveAgent),
    };

    let executor = HostExecutor;

    let master = SimpleState { counter: 0 };
    let tasks = vec![
        Task::new("inc-A", "3"),
        Task::new("inc-B", "5"),
        Task::new("root", "1"),
    ];

    let new_master = dax_run_sync(
        &agent,
        &executor,
        master,
        tasks,
        SplitStrategy::SemanticRouting,
        CollapseStrategy::Sequential,
        |s, _| s.clone(),
    );

    println!("Final master state (sync): {:?}", new_master);
}


