//! Example 15: Model fallback chain.
//!
//! Demonstrates:
//! - `ModelFallback` — ordered list of backup models tried when the primary fails
//! - `AgentEvent::ModelFallback` — event emitted on failover
//! - Tracing integration to log the failover warning
//! - A deliberately invalid primary API key to trigger fallback in the demo
//!
//! WARNING: `ModelFallback` masks errors from the primary model. Use it only
//! when you have a trustworthy secondary you are willing to pay for. Do NOT
//! use it to silently swallow authentication failures in production; add
//! monitoring on the `ModelFallback` event instead.
//!
//! ## When to use
//! - Multi-provider redundancy (e.g. primary provider outage)
//! - Rate-limit escape valve with a slower/cheaper secondary
//!
//! ## When NOT to use
//! - To hide misconfigured API keys (this example does so only for demo purposes)
//! - When you need hard failure on any error
//!
//! # Run
//!
//! ```text
//! cargo run -p model-fallback-basic
//! ```
//!
//! # Requires
//!
//! - `ANTHROPIC_API_KEY` (used for the fallback model)

use std::sync::Arc;

use swink_agent::prelude::*;
use swink_agent_adapters::AnthropicStreamFn;
use swink_agent_adapters::OpenAiStreamFn;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // Step 1: Initialize tracing (this example demonstrates observability).
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("warn".parse()?))
        .init();

    // Step 2: Build the primary stream function — intentionally uses an invalid
    //         API key so the request fails and triggers fallback.
    let bad_primary = OpenAiStreamFn::new(
        "https://api.openai.com",
        "invalid-key-to-trigger-fallback",
    );
    let primary_spec = ModelSpec::new("openai", "gpt-4o-mini");

    // Step 3: Build the fallback stream function — the real Anthropic connection.
    let fallback_key = std::env::var("ANTHROPIC_API_KEY")
        .map_err(|_| "ANTHROPIC_API_KEY environment variable not set")?;
    let fallback_stream = AnthropicStreamFn::new(
        "https://api.anthropic.com",
        fallback_key,
    );
    let fallback_spec = ModelSpec::new("anthropic", "claude-haiku-4-5-20251001");

    // Step 4: Build the ModelFallback chain.
    //
    // The chain is tried in order: primary fails → fallback is tried.
    let fallback = ModelFallback::new(vec![
        (
            fallback_spec.clone(),
            Arc::new(fallback_stream) as Arc<dyn StreamFn>,
        ),
    ]);

    // Step 5: Build the primary ModelConnection for AgentOptions.
    let primary_conn = ModelConnection::new(
        primary_spec,
        Arc::new(bad_primary) as Arc<dyn StreamFn>,
    );
    let connections = ModelConnections::new(primary_conn, vec![]);

    // Step 6: Build agent with fallback chain and event forwarder.
    let options = AgentOptions::from_connections("You are a helpful assistant.", connections)
        .with_model_fallback(fallback)
        .with_event_forwarder(|event| {
            if let AgentEvent::ModelFallback { from_model, to_model } = event {
                tracing::warn!(
                    from = %from_model.model_id,
                    to = %to_model.model_id,
                    "Model failover triggered — primary unavailable, switching to fallback"
                );
                println!(
                    "[FALLOVER] {} → {}",
                    from_model.model_id, to_model.model_id
                );
            }
        });

    let mut agent = Agent::new(options);

    // Step 7: Run a prompt — the primary will fail, fallback will answer.
    println!("Sending prompt (primary model has invalid key — fallback expected)...");
    let result = agent.prompt_text("What is 2+2?").await?;
    println!("Assistant: {}", result.assistant_text());
    println!("\nNote: the [FALLOVER] line above confirms the fallback model was used.");

    Ok(())
}
