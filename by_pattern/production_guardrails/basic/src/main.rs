//! Example: production-grade guarded agent with all guardrail layers.
//!
//! Demonstrates:
//! - BudgetPolicy (PreTurnPolicy) — stops when cost cap is exceeded
//! - MaxTurnsPolicy (PreTurnPolicy) — stops after N turns
//! - PromptInjectionGuard (PreTurnPolicy + PostTurnPolicy) — injection detection
//! - PiiRedactor (PostTurnPolicy) — redacts PII from assistant responses
//! - ContentFilter (PostTurnPolicy) — keyword blocklist
//! - LoopDetectionPolicy (PostTurnPolicy) — detects repeated tool patterns
//! - AuditLogger (PostTurnPolicy) — JSONL audit trail
//! - ToolMiddleware::with_logging — logs tool invocations
//! - MetricsCollector — prints cost and turn stats

use std::sync::Arc;

use swink_agent::prelude::*;
use swink_agent::{ApprovalMode, MetricsFuture, ToolMiddleware, TurnMetrics};
use swink_agent_adapters::build_remote_connection_for_model;
use swink_agent_policies::{
    AuditLogger, BudgetPolicy, ContentFilter, JsonlAuditSink, LoopDetectionPolicy, MaxTurnsPolicy,
    PiiRedactor, PromptInjectionGuard,
};

// ─── Custom metrics collector ────────────────────────────────────────────────

struct PrintMetrics;

impl MetricsCollector for PrintMetrics {
    fn on_metrics<'a>(&'a self, metrics: &'a TurnMetrics) -> MetricsFuture<'a> {
        Box::pin(async move {
            println!(
                "[metrics] turn={} cost=${:.6} tokens_in={} tokens_out={} tools={}",
                metrics.turn_index,
                metrics.cost.total,
                metrics.usage.input,
                metrics.usage.output,
                metrics.tool_executions.len(),
            );
        })
    }
}

// ─── Main ────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // ── 1. Build a simple customer lookup tool ───────────────────────────────

    let raw_tool = FnTool::new(
        "lookup_customer",
        "Customer Lookup",
        "Look up a customer record by ID.",
    )
    .with_execute_simple(|args, _cancel| async move {
        // Returns data with a fake SSN to exercise PiiRedactor.
        let id = args
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        AgentToolResult::text(format!(
            "Customer: John Doe, SSN: 555-22-4444, balance: ${id}00"
        ))
    })
    .into_tool();

    // ── 2. Wrap the tool with logging middleware ──────────────────────────────

    let logged_tool = Arc::new(ToolMiddleware::with_logging(
        raw_tool,
        |name, _id, is_start| {
            if is_start {
                println!("[middleware] -> tool='{name}' starting");
            } else {
                println!("[middleware] <- tool='{name}' done");
            }
        },
    ));

    // ── 3. Build model connection ────────────────────────────────────────────

    let connection = build_remote_connection_for_model("claude-haiku-4-5-20251001")?;
    let connections = ModelConnections::new(connection, vec![]);

    // ── 4. Compose all policies ───────────────────────────────────────────────
    //
    // Trait placements:
    //   PreTurnPolicy:  BudgetPolicy, MaxTurnsPolicy, PromptInjectionGuard
    //   PostTurnPolicy: PromptInjectionGuard (also), PiiRedactor, ContentFilter,
    //                   LoopDetectionPolicy, AuditLogger

    let audit_path = "/tmp/guardrails-audit.jsonl";

    let options = AgentOptions::from_connections(
        "You are a customer service assistant. \
         You have access to a customer lookup tool.",
        connections,
    )
    .with_tools(vec![logged_tool])
    .with_approval_mode(ApprovalMode::Bypassed)
    // PreTurn policies
    .with_pre_turn_policy(BudgetPolicy::new().max_cost(1.0))
    .with_pre_turn_policy(MaxTurnsPolicy::new(10))
    .with_pre_turn_policy(PromptInjectionGuard::new())
    // PostTurn policies
    .with_post_turn_policy(PromptInjectionGuard::new())
    .with_post_turn_policy(PiiRedactor::new())
    .with_post_turn_policy(ContentFilter::new().with_keyword("password"))
    .with_post_turn_policy(LoopDetectionPolicy::new(3))
    .with_post_turn_policy(AuditLogger::new(JsonlAuditSink::new(audit_path)))
    // Metrics
    .with_metrics_collector(PrintMetrics);

    // ── 5. Run prompts ───────────────────────────────────────────────────────

    let mut agent = Agent::new(options);

    for prompt in [
        "Look up customer ID 42.",
        "What's the balance for customer 99?",
    ] {
        println!("\n>>> {prompt}");
        let result = agent.prompt_text(prompt).await?;
        println!("Assistant: {}", result.assistant_text());
    }

    println!("\nAudit log written to: {audit_path}");

    Ok(())
}
