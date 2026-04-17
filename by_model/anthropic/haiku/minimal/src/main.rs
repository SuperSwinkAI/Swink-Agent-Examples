//! Minimal real-provider agent — ~30 lines, no TUI.
//!
//! Uses `build_remote_connection_for_model` to connect by model ID string
//! rather than a preset key. Useful when you want to pass the model ID
//! dynamically (e.g. from config or CLI args).
//!
//! For hardcoded provider examples, prefer the preset-key approach in
//! `by_model/anthropic/haiku/basic` instead.
//!
//! Run: `cargo run`
//! Requires: `ANTHROPIC_API_KEY`

use swink_agent::{Agent, AgentMessage, AgentOptions, ContentBlock, LlmMessage, ModelConnections};
use swink_agent_adapters::build_remote_connection_for_model;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // Build a connection by model ID string (reads the API key from env).
    let connection = build_remote_connection_for_model("claude-haiku-4-5-20251001")?;

    let connections = ModelConnections::new(connection, vec![]);
    let options = AgentOptions::from_connections("You are a helpful assistant.", connections);

    let mut agent = Agent::new(options);

    for prompt in [
        "We're passing your model ID as a runtime string instead of a compile-time constant. What operational benefit does that give a deployment team managing multiple LLM providers?",
        "Give me a concrete Rust struct — with serde derives — that would let a team configure model ID, provider, and base URL from a TOML file.",
    ] {
        println!(">>> {prompt}");
        let result = agent.prompt_text(prompt).await?;
        for msg in &result.messages {
            if let AgentMessage::Llm(LlmMessage::Assistant(a)) = msg {
                println!("{}\n", ContentBlock::extract_text(&a.content));
            }
        }
    }
    Ok(())
}
