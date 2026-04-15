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
    let result = agent.prompt_text("What is Rust?").await?;

    for msg in &result.messages {
        if let AgentMessage::Llm(LlmMessage::Assistant(a)) = msg {
            println!("{}", ContentBlock::extract_text(&a.content));
        }
    }
    Ok(())
}
