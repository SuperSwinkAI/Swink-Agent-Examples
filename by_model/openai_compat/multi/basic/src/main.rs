//! Demonstrates multiple OpenAI-compatible backends via Swink-Agent.
//!
//! Pass a backend name as the first argument (default: "lmstudio"):
//!   cargo run -- lmstudio   # local LM Studio server (default)
//!   cargo run -- azure      # Azure OpenAI endpoint
//!   cargo run -- xai        # xAI / Grok API

use std::sync::Arc;

use swink_agent::{
    Agent, AgentMessage, AgentOptions, ContentBlock, LlmMessage, ModelConnection, ModelConnections,
    ModelSpec,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let backend = std::env::args().nth(1).unwrap_or_else(|| "lmstudio".to_string());
    println!("Backend: {backend}");

    let connection: ModelConnection = match backend.as_str() {
        "azure" => {
            use swink_agent_adapters::{AzureAuth, AzureStreamFn};

            let base_url = std::env::var("AZURE_BASE_URL")?;
            let api_key = std::env::var("AZURE_API_KEY")?;
            let stream_fn = AzureStreamFn::new(base_url, AzureAuth::ApiKey(api_key));
            let spec = ModelSpec::new("azure", "gpt-4o");
            ModelConnection::new(spec, Arc::new(stream_fn))
        }
        "xai" => {
            use swink_agent_adapters::XAiStreamFn;

            let api_key = std::env::var("XAI_API_KEY")?;
            let stream_fn = XAiStreamFn::new("https://api.x.ai", api_key);
            let spec = ModelSpec::new("xai", "grok-4.20-0309-non-reasoning");
            ModelConnection::new(spec, Arc::new(stream_fn))
        }
        _ => {
            // "lmstudio" or any unrecognised value falls back to LM Studio
            use swink_agent_adapters::OpenAiStreamFn;

            let base_url = std::env::var("LM_STUDIO_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:1234/v1".to_string());
            let model_name = std::env::var("LM_STUDIO_MODEL")
                .unwrap_or_else(|_| "local-model".to_string());
            let stream_fn = OpenAiStreamFn::new(base_url, "lm-studio");
            let spec = ModelSpec::new("openai", model_name);
            ModelConnection::new(spec, Arc::new(stream_fn))
        }
    };

    let connections = ModelConnections::new(connection, vec![]);
    let options = AgentOptions::from_connections("You are a helpful assistant.", connections);

    let mut agent = Agent::new(options);

    let prompt = "Explain what 'OpenAI-compatible' means in one paragraph.";
    println!(">>> {prompt}");
    let result = agent.prompt_text(prompt).await?;
    for msg in &result.messages {
        if let AgentMessage::Llm(LlmMessage::Assistant(a)) = msg {
            println!("{}\n", ContentBlock::extract_text(&a.content));
        }
    }

    Ok(())
}
