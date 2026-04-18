//! Demonstrates the Google Gemini adapter for Swink-Agent.
//!
//! Uses `build_remote_connection_for_model` to connect to Gemini Flash via
//! the `GEMINI_API_KEY` environment variable, with clean error handling on
//! connection failure (no fallback to another provider).

use swink_agent::{Agent, AgentMessage, AgentOptions, ContentBlock, LlmMessage, ModelConnections};
use swink_agent_adapters::build_remote_connection_for_model;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let connection = match build_remote_connection_for_model("gemini-3-flash-preview") {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error connecting to Gemini: {e}");
            return Err(e.into());
        }
    };

    let connections = ModelConnections::new(connection, vec![]);
    let options = AgentOptions::from_connections("You are a helpful assistant.", connections);

    let mut agent = Agent::new(options);

    for prompt in [
        "What is Rust's ownership model in one sentence?",
        "Give one example of a zero-cost abstraction in Rust.",
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
