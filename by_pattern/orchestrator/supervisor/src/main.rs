//! Supervised planner/executor hierarchy. DefaultSupervisor restarts either
//! agent on retryable errors up to three times. The orchestrator owns the
//! agents; callers use OrchestratedHandle.

use swink_agent::{AgentOptions, AgentOrchestrator, DefaultSupervisor, ModelConnections};
use swink_agent_adapters::build_remote_connection_for_model;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let connection = build_remote_connection_for_model("claude-haiku-4-5-20251001")?;

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

    let planner = orchestrator.spawn("planner")?;
    let plan_result = planner
        .send_message("Plan how to write a blog post about async Rust.")
        .await?;
    println!("Planner output:\n{}\n", plan_result.assistant_text());

    let executor = orchestrator.spawn("executor")?;
    let exec_result = executor
        .send_message("Execute step 1: write an outline for a blog post about async Rust.")
        .await?;
    println!("Executor output:\n{}", exec_result.assistant_text());

    Ok(())
}
