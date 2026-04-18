use std::sync::Arc;

use swink_agent::{
    Agent, AgentMessage, AgentOptions, ContentBlock, LlmMessage, ModelConnection, ModelConnections,
    ModelSpec,
};
use swink_agent_adapters::OllamaStreamFn;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Ollama runs locally — no API key needed, just `ollama serve`.
    // Pull the model first: `ollama pull gemma4:31b`
    let stream_fn = Arc::new(OllamaStreamFn::new("http://localhost:11434"));
    let model = ModelSpec::new("local", "gemma4:31b");
    let connection = ModelConnection::new(model, stream_fn);
    let connections = ModelConnections::new(connection, vec![]);
    let options = AgentOptions::from_connections("You are a helpful assistant.", connections);

    let mut agent = Agent::new(options);

    for prompt in [
        "You're running locally via Ollama — no data leaves this machine. What privacy-sensitive use cases does that unlock that a cloud API can't offer?",
        "A startup building a code-review tool is deciding between running you locally vs. calling a cloud API. Walk through the tradeoffs on latency, cost, and maintenance burden.",
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
