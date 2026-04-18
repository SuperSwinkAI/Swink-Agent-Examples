//! Example: 3-stage research pipeline using `swink-agent-patterns`.
//!
//! Stages:
//!  1. **outline** — generates a 5-point outline for the given topic
//!  2. **draft**   — writes a 3-paragraph summary from the outline
//!  3. **critic**  — reviews the draft; loops until output starts with
//!                   "APPROVED" or the iteration cap (2) is hit
//!
//! Run:
//!   cargo run -- "the benefits of Rust's type system for systems programming"

use std::sync::Arc;

use swink_agent::{Agent, AgentOptions, ModelConnections};
use swink_agent_adapters::build_remote_connection_for_model;
use swink_agent_patterns::{
    ExitCondition, Pipeline, PipelineExecutor, PipelineRegistry, SimpleAgentFactory,
};
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let topic = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "the benefits of Rust's type system for systems programming".into());

    println!("Research topic: {topic}\n");

    let connection = build_remote_connection_for_model("claude-haiku-4-5-20251001")?;

    // ── Agent factory ──────────────────────────────────────────────────────────
    // Register builder functions by name. The pipeline executor calls these
    // to create a fresh agent for each step.
    let mut factory = SimpleAgentFactory::new();

    factory.register("outline", {
        let conn = connection.clone();
        move || {
            Agent::new(AgentOptions::from_connections(
                "Generate a concise 5-point outline for the given research topic. \
                 Number each point.",
                ModelConnections::new(conn.clone(), vec![]),
            ))
        }
    });

    factory.register("draft", {
        let conn = connection.clone();
        move || {
            Agent::new(AgentOptions::from_connections(
                "Write a 3-paragraph research summary based on the outline provided. \
                 Be informative and well-structured.",
                ModelConnections::new(conn.clone(), vec![]),
            ))
        }
    });

    factory.register("critic", {
        let conn = connection.clone();
        move || {
            Agent::new(AgentOptions::from_connections(
                "Review the draft. If it is well-structured and covers the topic clearly, \
                 begin your response with exactly 'APPROVED'. Otherwise, suggest specific \
                 improvements without rewriting the draft.",
                ModelConnections::new(conn.clone(), vec![]),
            ))
        }
    });

    // ── Pipeline definitions ───────────────────────────────────────────────────
    // Sequential pipeline: outline → draft.
    // Steps reference registered agent names; output from each step is passed
    // as input to the next.
    let main_pipeline = Pipeline::sequential_with_context(
        "research",
        vec!["outline".into(), "draft".into()],
    );

    // Loop pipeline: critic reviews the draft, exiting when the output
    // contains "APPROVED" (case-sensitive prefix) or after 2 iterations.
    let review_loop = Pipeline::loop_with_max(
        "review",
        "critic",
        ExitCondition::output_contains("APPROVED").map_err(|e| e)?,
        2,
    );

    // ── Registry ───────────────────────────────────────────────────────────────
    let registry = Arc::new(PipelineRegistry::new());
    registry.register(main_pipeline.clone());
    registry.register(review_loop.clone());

    // ── Execution ──────────────────────────────────────────────────────────────
    let executor = PipelineExecutor::new(Arc::new(factory), Arc::clone(&registry));
    let ct = CancellationToken::new();

    // Stage 1 + 2: outline and draft
    println!("Running outline → draft pipeline…");
    let draft_output = executor
        .run(main_pipeline.id(), topic.clone(), ct.clone())
        .await?;

    println!("--- Draft ---");
    println!("{}\n", draft_output.final_response);

    for step in &draft_output.steps {
        println!(
            "[{}] {:?} ({} input tokens)",
            step.agent_name,
            step.duration,
            step.usage.input,
        );
    }

    // Stage 3: critic review loop
    println!("\nRunning critic review loop…");
    let review_output = executor
        .run(
            review_loop.id(),
            draft_output.final_response.clone(),
            ct.clone(),
        )
        .await?;

    println!("--- Critic ---");
    println!("{}", review_output.final_response);

    Ok(())
}
