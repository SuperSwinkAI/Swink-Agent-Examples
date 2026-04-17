use swink_agent::{Agent, AgentMessage, AgentOptions, ContentBlock, LlmMessage, ModelConnections};
use swink_agent_local_llm::default_local_connection;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let connection = default_local_connection()?;
    let connections = ModelConnections::new(connection, vec![]);

    let options = AgentOptions::from_connections(
        "You are a helpful assistant running entirely on-device.",
        connections,
    );

    let mut agent = Agent::new(options);

    for prompt in [
        "You're SmolLM3 — built for edge hardware. What's the smallest device class you can run on?",
        "Why would a mobile app team pick you over a larger local model like Gemma 4?",
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
