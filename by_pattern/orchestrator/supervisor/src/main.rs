//! Example: Supervised multi-agent orchestration with parent/child hierarchy.
//!
//! This example sets up two agents in a hierarchy:
//! - `planner`: decomposes a high-level task into steps
//! - `executor`: a child of `planner` that executes a single step
//!
//! A `DefaultSupervisor` is attached so that either agent can be restarted
//! automatically on retryable errors (e.g. throttling) up to 3 times.
//!
//! # How this differs from `swarm/basic`
//!
//! `swarm/basic` uses peer-to-peer agent *transfer* — one agent hands the
//! conversation off to another at runtime via a `TransferToAgentTool`. There
//! is no parent/child relationship and no automatic restart on failure.
//!
//! `orchestrator/supervisor` uses a *hierarchical* model: agents are
//! registered with explicit parent links, and a supervisor policy decides
//! whether to restart, stop, or escalate after each failure. The orchestrator
//! retains ownership of all agents and spawns them on demand; callers interact
//! through `OrchestratedHandle`, not through a shared registry.

use swink_agent::{
    AgentOptions, AgentOrchestrator, DefaultSupervisor, ModelConnections,
};
use swink_agent_adapters::build_remote_connection_for_model;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let connection = build_remote_connection_for_model("claude-haiku-4-5-20251001")?;

    // Build the orchestrator with a supervisor that allows up to 3 restarts.
    // `add_agent` and `add_child` take `&mut self`, so we build the orchestrator
    // imperatively rather than with a fluent builder.
    let mut orchestrator = AgentOrchestrator::new()
        .with_supervisor(DefaultSupervisor::new(3))
        .with_max_restarts(3);

    orchestrator.add_agent("planner", {
        let conn = connection.clone();
        move || {
            AgentOptions::from_connections(
                "You are a task planner. Break down the user's request into 2-3 clear, \
                 numbered steps. Be concise.",
                ModelConnections::new(conn.clone(), vec![]),
            )
        }
    });

    // `executor` is a child of `planner` in the hierarchy. The orchestrator
    // tracks this relationship so a supervisor can escalate or stop the whole
    // subtree when a child fails beyond its restart budget.
    orchestrator.add_child("executor", "planner", {
        let conn = connection.clone();
        move || {
            AgentOptions::from_connections(
                "You are a task executor. Complete the single step you are given. \
                 Be concise and direct.",
                ModelConnections::new(conn.clone(), vec![]),
            )
        }
    });

    // Spawn the planner and ask it to decompose a task.
    // `send_message` is async and returns the full agent result directly.
    let planner = orchestrator.spawn("planner")?;
    let plan_result = planner
        .send_message("Plan how to write a blog post about async Rust.")
        .await?;
    println!("Planner output:\n{}\n", plan_result.assistant_text());

    // Spawn the executor independently and ask it to carry out one step.
    let executor = orchestrator.spawn("executor")?;
    let exec_result = executor
        .send_message("Execute step 1: write an outline for a blog post about async Rust.")
        .await?;
    println!("Executor output:\n{}", exec_result.assistant_text());

    Ok(())
}
