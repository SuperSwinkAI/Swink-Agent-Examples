//! Example 6: OpenTelemetry stack integration.
//!
//! Demonstrates:
//! - Initializing a tracing layer that exports spans via OTLP to an OTel backend
//! - A custom `MetricsCollector` that prints per-turn cost and duration
//! - A custom `TokenCounter` backed by `tiktoken-rs` for accurate token budgeting
//! - Running 3 sequential prompts and printing metrics after each turn
//!
//! # Run
//!
//! ```text
//! # Optional: start a local OTel collector
//! docker run -p 4317:4317 otel/opentelemetry-collector:latest
//!
//! cargo run -p observability-otel-stack
//! ```
//!
//! # Requires
//!
//! - `ANTHROPIC_API_KEY`
//! - Optional: `OTEL_EXPORTER_OTLP_ENDPOINT` (default: `http://localhost:4317`)

use std::sync::atomic::{AtomicU32, Ordering};

use swink_agent::prelude::*;
use swink_agent::{MetricsFuture, TurnMetrics};
use swink_agent::{OtelInitConfig, init_otel_layer};
use swink_agent_adapters::build_remote_connection_for_model;
use tracing_subscriber::layer::SubscriberExt as _;
use tracing_subscriber::util::SubscriberInitExt as _;

// ─── PrintMetrics ────────────────────────────────────────────────────────────

struct PrintMetrics {
    turn_count: AtomicU32,
}

impl MetricsCollector for PrintMetrics {
    fn on_metrics<'a>(&'a self, metrics: &'a TurnMetrics) -> MetricsFuture<'a> {
        Box::pin(async move {
            let n = self.turn_count.fetch_add(1, Ordering::SeqCst) + 1;
            println!(
                "[metrics] turn={n} index={} llm={:.2}s total={:.2}s cost=${:.6} tokens_in={} tokens_out={}",
                metrics.turn_index,
                metrics.llm_call_duration.as_secs_f64(),
                metrics.turn_duration.as_secs_f64(),
                metrics.cost.total,
                metrics.usage.input,
                metrics.usage.output,
            );
            if !metrics.tool_executions.is_empty() {
                for tool in &metrics.tool_executions {
                    println!(
                        "[metrics]   tool='{}' duration={:.2}s success={}",
                        tool.tool_name,
                        tool.duration.as_secs_f64(),
                        tool.success,
                    );
                }
            }
        })
    }
}

// ─── TiktokenCounter ─────────────────────────────────────────────────────────

struct TiktokenCounter(tiktoken_rs::CoreBPE);

impl TokenCounter for TiktokenCounter {
    fn count_tokens(&self, message: &AgentMessage) -> usize {
        // Extract all text from the message and count tokens.
        match message {
            AgentMessage::Llm(llm_msg) => {
                let text: String = match llm_msg {
                    LlmMessage::User(m) => m
                        .content
                        .iter()
                        .filter_map(|b| {
                            if let ContentBlock::Text { text } = b {
                                Some(text.as_str())
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(" "),
                    LlmMessage::Assistant(m) => m
                        .content
                        .iter()
                        .filter_map(|b| {
                            if let ContentBlock::Text { text } = b {
                                Some(text.as_str())
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(" "),
                    LlmMessage::ToolResult(m) => m
                        .content
                        .iter()
                        .filter_map(|b| {
                            if let ContentBlock::Text { text } = b {
                                Some(text.as_str())
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(" "),
                };
                self.0.encode_with_special_tokens(&text).len()
            }
            AgentMessage::Custom(_) => 100,
        }
    }
}

// ─── Main ────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // Step 1: Initialize OTel tracing layer.
    let endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4317".into());
    println!("[otel] Sending traces to: {endpoint}");

    let otel_layer = init_otel_layer(OtelInitConfig {
        service_name: "swink-otel-example".into(),
        endpoint: Some(endpoint),
    });
    tracing_subscriber::registry().with(otel_layer).init();

    // Step 2: Build connection.
    let connection = build_remote_connection_for_model("claude-haiku-4-5-20251001")?;
    let connections = ModelConnections::new(connection, vec![]);

    // Step 3: Build agent with metrics collector and tiktoken counter.
    let options = AgentOptions::from_connections(
        "You are a concise and helpful assistant.",
        connections,
    )
    .with_metrics_collector(PrintMetrics {
        turn_count: AtomicU32::new(0),
    })
    .with_token_counter(TiktokenCounter(tiktoken_rs::cl100k_base()?));

    let mut agent = Agent::new(options);

    // Step 4: Run 3 prompts, metrics are printed automatically after each turn.
    let prompts = [
        "What is the capital of France?",
        "Name three famous Impressionist painters.",
        "Summarize what we've discussed so far.",
    ];

    for prompt in prompts {
        println!("\n>>> {prompt}");
        let result = agent.prompt_text(prompt).await?;
        println!("Assistant: {}", result.assistant_text());
    }

    println!("\n[otel] Traces exported to the OTLP endpoint.");
    println!("[otel] Use Jaeger, Zipkin, or Grafana Tempo to view them.");

    Ok(())
}
