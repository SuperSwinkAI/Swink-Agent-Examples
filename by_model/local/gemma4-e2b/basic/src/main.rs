use std::sync::Arc;

use swink_agent::{
    Agent, AgentMessage, AgentOptions, ContentBlock, LlmMessage, ModelConnection, ModelConnections,
    model_catalog,
};
use swink_agent_local_llm::{LocalModel, LocalStreamFn, ModelPreset};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let preset = model_catalog()
        .preset("local", "gemma4_e2b")
        .ok_or("gemma4_e2b not found in catalog")?;
    let model_spec = preset.model_spec();
    let model = LocalModel::from_preset(ModelPreset::Gemma4E2B);
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
        "No network, no cloud — you're running fully on-device. What's one thing you can do that a cloud API fundamentally can't?",
        "Name the single biggest limitation a developer should plan around when shipping you in a product.",
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
