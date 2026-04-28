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
use swink_agent_eval::{
    AgentFactory, BudgetConstraints, EvalCase, EvalRunner, EvalSet, ResponseCriteria,
};
use swink_agent_eval::EvalError;
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
        EvalCase {
            id: "case-1-arithmetic".into(),
            name: "Basic arithmetic".into(),
            description: Some("Agent must answer 2+2 = 4".into()),
            system_prompt: "You are a helpful assistant. Answer concisely.".into(),
            user_messages: vec!["What is 2+2?".into()],
            expected_trajectory: None,
            expected_response: Some(ResponseCriteria::Contains {
                substring: "4".into(),
            }),
            expected_assertion: None,
            expected_interactions: None,
            few_shot_examples: vec![],
            budget: None,
            evaluators: vec![],
            metadata: serde_json::Value::Null,
            attachments: vec![],
            session_id: None,
            expected_environment_state: None,
            expected_tool_intent: None,
            semantic_tool_selection: false,
            state_capture: None,
        },
        // Case 2: Response must mention a key concept.
        EvalCase {
            id: "case-2-rust-ownership".into(),
            name: "Rust ownership concept".into(),
            description: Some("Agent must explain ownership in Rust".into()),
            system_prompt: "You are an expert Rust programming tutor.".into(),
            user_messages: vec!["Briefly explain Rust ownership in one sentence.".into()],
            expected_trajectory: None,
            expected_response: Some(ResponseCriteria::Contains {
                substring: "owner".into(),
            }),
            expected_assertion: None,
            expected_interactions: None,
            few_shot_examples: vec![],
            budget: None,
            evaluators: vec![],
            metadata: serde_json::Value::Null,
            attachments: vec![],
            session_id: None,
            expected_environment_state: None,
            expected_tool_intent: None,
            semantic_tool_selection: false,
            state_capture: None,
        },
        // Case 3: Budget-constrained case.
        EvalCase {
            id: "case-3-budget".into(),
            name: "Budget-constrained response".into(),
            description: Some("Agent must answer within tight turn and cost budgets".into()),
            system_prompt: "You are a helpful assistant. Be extremely brief.".into(),
            user_messages: vec!["Name one planet in our solar system.".into()],
            expected_trajectory: None,
            expected_response: None,
            expected_assertion: None,
            expected_interactions: None,
            few_shot_examples: vec![],
            budget: Some(BudgetConstraints {
                max_turns: Some(3),
                max_cost: Some(0.10),
                max_input: Some(10_000),
                max_output: None,
            }),
            evaluators: vec![],
            metadata: serde_json::Value::Null,
            attachments: vec![],
            session_id: None,
            expected_environment_state: None,
            expected_tool_intent: None,
            semantic_tool_selection: false,
            state_capture: None,
        },
    ];

    let eval_set = EvalSet {
        id: "basic-eval-set".into(),
        name: "Basic Eval Set".into(),
        description: Some("Three representative eval cases".into()),
        cases,
    };

    // Build the factory.
    let connection = build_remote_connection_for_model("claude-haiku-4-5-20251001")?;
    let factory = ExampleFactory { connection };

    // Run the eval set.
    let runner = EvalRunner::with_defaults();
    println!("Running eval set '{}' ({} cases)...", eval_set.name, eval_set.cases.len());

    let result = runner.run_set(&eval_set, &factory).await?;

    // Print results.
    println!(
        "\nResults: {}/{} passed",
        result.summary.passed, result.summary.total_cases
    );
    for case_result in &result.case_results {
        println!(
            "  {} — {:?}",
            case_result.case_id, case_result.verdict
        );
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
