use std::sync::Arc;

use swink_agent::{
    Agent, AgentMessage, AgentOptions, ContentBlock, LlmMessage, ModelConnection, ModelConnections,
    model_catalog,
};
use swink_agent_local_llm::{LocalModel, LocalStreamFn, ModelPreset};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let preset = model_catalog()
        .preset("local", "gemma4_e4b")
        .ok_or("gemma4_e4b not found in catalog")?;
    let model_spec = preset.model_spec();
    let model = LocalModel::from_preset(ModelPreset::Gemma4E4B);
    let connection = ModelConnection::new(
        model_spec,
        Arc::new(LocalStreamFn::new(Arc::new(model))),
    );

    let connections = ModelConnections::new(connection, vec![]);
    let options = AgentOptions::from_connections(
        "You are a helpful assistant running entirely on-device.",
        connections,
    );

    let mut agent = Agent::new(options);

    for prompt in [
        "In one sentence, what is the capital of France?",
        "What is the population of that city?",
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
