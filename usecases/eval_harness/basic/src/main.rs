//! Example 14: Basic evaluation harness.
//!
//! Demonstrates:
//! - Defining `EvalCase` scenarios with expected responses and budget constraints
//! - Implementing `AgentFactory` to wire up an `Agent` per eval case
//! - Attaching `AuditLogger` as a `PostTurnPolicy` inside the factory
//! - Running an `EvalSet` and printing pass/fail results
//! - Writing the full results as JSON to `/tmp/eval-results.json`
//!
//! # Run
//!
//! ```text
//! cargo run -p eval-harness-basic
//! ```
//!
//! # Requires
//!
//! - `ANTHROPIC_API_KEY`

use swink_agent::prelude::*;
use swink_agent_adapters::build_remote_connection_for_model;
use swink_agent_eval::EvalError;
use swink_agent_eval::{
    AgentFactory, BudgetConstraints, EvalCase, EvalRunner, EvalSet, ResponseCriteria,
};
use swink_agent_policies::{AuditLogger, JsonlAuditSink};
use tokio_util::sync::CancellationToken;

// ─── ExampleFactory ──────────────────────────────────────────────────────────

struct ExampleFactory {
    connection: ModelConnection,
}

impl AgentFactory for ExampleFactory {
    fn create_agent(&self, case: &EvalCase) -> Result<(Agent, CancellationToken), EvalError> {
        let ct = CancellationToken::new();
        let connections = ModelConnections::new(self.connection.clone(), vec![]);
        let audit_path = format!("/tmp/eval-audit-{}.jsonl", case.id);
        let options = AgentOptions::from_connections(&case.system_prompt, connections)
            .with_post_turn_policy(AuditLogger::new(JsonlAuditSink::new(audit_path)));
        Ok((Agent::new(options), ct))
    }
}

// ─── Main ────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // Build the eval cases.
    let cases = vec![
        // Case 1: Basic arithmetic response check.
        EvalCase::new(
            "case-1-arithmetic",
            "Basic arithmetic",
            "You are a helpful assistant. Answer concisely.",
            vec!["What is 2+2?".into()],
        )
        .with_description("Agent must answer 2+2 = 4")
        .with_expected_response(ResponseCriteria::Contains {
            substring: "4".into(),
        }),
        // Case 2: Response must mention a key concept.
        EvalCase::new(
            "case-2-rust-ownership",
            "Rust ownership concept",
            "You are an expert Rust programming tutor.",
            vec!["Briefly explain Rust ownership in one sentence.".into()],
        )
        .with_description("Agent must explain ownership in Rust")
        .with_expected_response(ResponseCriteria::Contains {
            substring: "owner".into(),
        }),
        // Case 3: Budget-constrained case.
        EvalCase::new(
            "case-3-budget",
            "Budget-constrained response",
            "You are a helpful assistant. Be extremely brief.",
            vec!["Name one planet in our solar system.".into()],
        )
        .with_description("Agent must answer within tight turn and cost budgets")
        .with_budget(
            BudgetConstraints::default()
                .with_max_turns(3)
                .with_max_cost(0.10)
                .with_max_input(10_000),
        ),
    ];

    let eval_set = EvalSet::new("basic-eval-set", "Basic Eval Set", cases)
        .with_description("Three representative eval cases");

    // Build the factory.
    let connection = build_remote_connection_for_model("claude-haiku-4-5-20251001")?;
    let factory = ExampleFactory { connection };

    // Run the eval set.
    let runner = EvalRunner::with_defaults();
    println!(
        "Running eval set '{}' ({} cases)...",
        eval_set.name,
        eval_set.cases.len()
    );

    let result = runner.run_set(&eval_set, &factory).await?;

    // Print results.
    println!(
        "\nResults: {}/{} passed",
        result.summary.passed, result.summary.total_cases
    );
    for case_result in &result.case_results {
        println!("  {} — {:?}", case_result.case_id, case_result.verdict);
        for metric in &case_result.metric_results {
            if let Some(details) = &metric.details {
                println!(
                    "    [{}] score={:.2} — {}",
                    metric.evaluator_name, metric.score.value, details
                );
            }
        }
    }

    // Write full results as JSON.
    let json = serde_json::to_string_pretty(&result)?;
    std::fs::write("/tmp/eval-results.json", &json)?;
    println!("\nFull results written to /tmp/eval-results.json");
    println!("Per-case audit logs written to /tmp/eval-audit-<case-id>.jsonl");

    Ok(())
}
