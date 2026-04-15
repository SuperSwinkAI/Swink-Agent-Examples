use std::sync::Arc;

use swink_agent::{
    Agent, AgentMessage, AgentOptions, ContentBlock, LlmMessage, ModelConnection, ModelConnections,
    ModelSpec,
};
use swink_agent_adapters::OllamaStreamFn;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Ollama runs locally — no API key needed, just `ollama serve`.
    // Pull the model first: `ollama pull llama3:8b`
    let stream_fn = Arc::new(OllamaStreamFn::new("http://localhost:11434"));
    let model = ModelSpec::new("local", "llama3:8b");
    let connection = ModelConnection::new(model, stream_fn);
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
