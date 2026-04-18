//! Demonstrates two AWS Bedrock connection paths for Swink-Agent.
//!
//! Default (env-based): uses `build_remote_connection_for_model` which reads
//! AWS credentials automatically from environment variables.
//!
//! `--direct` flag: constructs `BedrockStreamFn` manually from explicit
//! env var reads, showing the lower-level adapter API.

use swink_agent::{Agent, AgentMessage, AgentOptions, ContentBlock, LlmMessage, ModelConnections};
use swink_agent_adapters::build_remote_connection_for_model;

const MODEL_ID: &str = "us.anthropic.claude-haiku-4-5-20251001-v1:0";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let use_direct = std::env::args().any(|a| a == "--direct");

    let connection = if use_direct {
        println!("Using direct BedrockStreamFn construction.");
        use std::sync::Arc;
        use swink_agent::{ModelConnection, ModelSpec};
        use swink_agent_adapters::BedrockStreamFn;

        let region = std::env::var("AWS_REGION")?;
        let access_key = std::env::var("AWS_ACCESS_KEY_ID")?;
        let secret_key = std::env::var("AWS_SECRET_ACCESS_KEY")?;
        let session_token = std::env::var("AWS_SESSION_TOKEN").ok();
        let stream_fn = BedrockStreamFn::new(region, access_key, secret_key, session_token);
        let spec = ModelSpec::new("bedrock", MODEL_ID);
        ModelConnection::new(spec, Arc::new(stream_fn))
    } else {
        println!("Using build_remote_connection_for_model (env-based).");
        build_remote_connection_for_model(MODEL_ID)?
    };

    let connections = ModelConnections::new(connection, vec![]);
    let options = AgentOptions::from_connections("You are a helpful assistant.", connections);

    let mut agent = Agent::new(options);

    let prompt = "What is Amazon Bedrock and why would you run LLMs on it instead of calling a provider directly?";
    println!(">>> {prompt}");
    let result = agent.prompt_text(prompt).await?;
    for msg in &result.messages {
        if let AgentMessage::Llm(LlmMessage::Assistant(a)) = msg {
            println!("{}\n", ContentBlock::extract_text(&a.content));
        }
    }

    Ok(())
}
