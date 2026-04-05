use swink_agent::{Agent, AgentMessage, AgentOptions, ContentBlock, LlmMessage, ModelConnections};
use swink_agent_adapters::{build_remote_connection, remote_preset_keys};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let connection = build_remote_connection(remote_preset_keys::anthropic::HAIKU_45)?;
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
