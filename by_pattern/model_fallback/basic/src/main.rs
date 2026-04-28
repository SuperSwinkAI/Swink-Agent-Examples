//! Example: Model fallback chain.
//!
//! Demonstrates:
//! - `ModelFallback` — ordered list of backup models tried when the primary fails
//! - `AgentEvent::ModelFallback` — event emitted on failover
//! - Tracing integration to log the failover warning
//! - A mock primary that returns a throttle error to trigger fallback
//!
//! WARNING: `ModelFallback` masks errors from the primary model. Use it only
//! when you have a trustworthy secondary you are willing to pay for. Add
//! monitoring on the `ModelFallback` event so you know when it fires.
//!
//! ## When to use
//! - Multi-provider redundancy (e.g. primary provider outage)
//! - Rate-limit escape valve with a slower/cheaper secondary
//!
//! ## When NOT to use
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

use std::pin::Pin;
use std::sync::Arc;

use futures::Stream;
use tokio_util::sync::CancellationToken;

use swink_agent::prelude::*;
use swink_agent_adapters::AnthropicStreamFn;
use tracing_subscriber::EnvFilter;

// ─── Throttled primary ──────────────────────────────────────────────────────

/// A `StreamFn` that always returns a throttle error, simulating a provider
/// that is rate-limiting or overloaded.
struct ThrottledStreamFn;

impl StreamFn for ThrottledStreamFn {
    fn stream<'a>(
        &'a self,
        _model: &'a ModelSpec,
        _context: &'a AgentContext,
        _options: &'a StreamOptions,
        _cancellation_token: CancellationToken,
    ) -> Pin<Box<dyn Stream<Item = AssistantMessageEvent> + Send + 'a>> {
        Box::pin(futures::stream::iter(vec![
            AssistantMessageEvent::error_throttled("rate limit exceeded — try again later"),
        ]))
    }
}

// ─── Main ───────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("warn".parse()?))
        .init();

    // Primary: always throttled.
    let primary_conn = ModelConnection::new(
        ModelSpec::new("openai", "gpt-5"),
        Arc::new(ThrottledStreamFn) as Arc<dyn StreamFn>,
    );
    let connections = ModelConnections::new(primary_conn, vec![]);

    // Fallback: real Anthropic connection.
    let fallback_key = std::env::var("ANTHROPIC_API_KEY")
        .map_err(|_| "ANTHROPIC_API_KEY environment variable not set")?;
    let fallback_stream = AnthropicStreamFn::new("https://api.anthropic.com", fallback_key);
    let fallback_spec = ModelSpec::new("anthropic", "claude-haiku-4-5-20251001");

    let fallback = ModelFallback::new(vec![(
        fallback_spec.clone(),
        Arc::new(fallback_stream) as Arc<dyn StreamFn>,
    )]);

    let options = AgentOptions::from_connections("You are a helpful assistant.", connections)
        .with_model_fallback(fallback)
        .with_event_forwarder(|event| {
            if let AgentEvent::ModelFallback {
                from_model,
                to_model,
            } = event
            {
                tracing::warn!(
                    from = %from_model.model_id,
                    to = %to_model.model_id,
                    "Model failover triggered"
                );
                println!(
                    "[FALLOVER] {} → {}",
                    from_model.model_id, to_model.model_id
                );
            }
        });

    let mut agent = Agent::new(options);

    println!("Sending prompt (primary model is throttled — fallback expected)...");
    let result = agent.prompt_text("What is 2+2?").await?;
    println!("Assistant: {}", result.assistant_text());
    println!("\nThe [FALLOVER] line above confirms the fallback model was used.");

    Ok(())
}
