use swink_agent::{
    Agent, AgentMessage, AgentOptions, AgentRegistry, ContentBlock, LlmMessage, ModelConnections,
};
use swink_agent_adapters::{build_remote_connection, remote_preset_keys};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let connection = build_remote_connection(remote_preset_keys::anthropic::HAIKU_45)?;

    // --- Triage agent: decides which specialist should handle the request ---
    let triage_conn = ModelConnections::new(connection.clone(), vec![]);
    let triage_options = AgentOptions::from_connections(
        "You are a triage agent. Route the user's request to the appropriate specialist. \
         Transfer to 'researcher' for factual questions or 'writer' for creative tasks.",
        triage_conn,
    );

    // --- Researcher agent ---
    let researcher_conn = ModelConnections::new(connection.clone(), vec![]);
    let researcher_options = AgentOptions::from_connections(
        "You are a research specialist. Answer factual questions concisely and accurately.",
        researcher_conn,
    );

    // --- Writer agent ---
    let writer_conn = ModelConnections::new(connection.clone(), vec![]);
    let writer_options = AgentOptions::from_connections(
        "You are a creative writing specialist. Help with creative tasks like stories and poems.",
        writer_conn,
    );

    // Register all agents so they can transfer between each other.
    let mut registry = AgentRegistry::new();
    registry.register("triage", Agent::new(triage_options));
    registry.register("researcher", Agent::new(researcher_options));
    registry.register("writer", Agent::new(writer_options));

    // Start from the triage agent — it will transfer as needed.
    let result = registry
        .prompt_text("triage", "Write me a haiku about Rust programming.")
        .await?;

    for msg in &result.messages {
        if let AgentMessage::Llm(LlmMessage::Assistant(a)) = msg {
            println!("{}", ContentBlock::extract_text(&a.content));
        }
    }
    Ok(())
}
