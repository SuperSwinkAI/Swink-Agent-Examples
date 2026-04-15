use std::sync::Arc;

use swink_agent::{
    Agent, AgentMessage, AgentOptions, AgentRegistry, ContentBlock, IntoTool, LlmMessage,
    ModelConnections, TransferToAgentTool,
};
use swink_agent_adapters::build_remote_connection_for_model;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let connection = build_remote_connection_for_model("claude-haiku-4-5-20251001")?;

    // Shared registry — must be Arc so TransferToAgentTool can hold a reference.
    let registry = Arc::new(AgentRegistry::new());

    // --- Triage agent: gets a TransferToAgentTool so it can hand off to specialists ---
    let transfer_tool = TransferToAgentTool::new(Arc::clone(&registry)).into_tool();
    let triage_conn = ModelConnections::new(connection.clone(), vec![]);
    let triage_options = AgentOptions::from_connections(
        "You are a triage agent. Route the user's request to the appropriate specialist. \
         Transfer to 'researcher' for factual questions or 'writer' for creative tasks.",
        triage_conn,
    )
    .with_tools(vec![transfer_tool]);

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

    registry.register("triage", Agent::new(triage_options));
    registry.register("researcher", Agent::new(researcher_options));
    registry.register("writer", Agent::new(writer_options));

    // Start from the triage agent, following any transfer signal to a specialist.
    let user_message = "Write me a haiku about Rust programming.";
    let mut current = "triage".to_string();
    let mut prompt: Option<String> = Some(user_message.to_string());

    loop {
        let agent_ref = registry
            .get(&current)
            .unwrap_or_else(|| panic!("agent '{current}' not registered"));
        let mut agent = agent_ref.lock().await;

        let result = agent
            .prompt_text(prompt.take().unwrap_or_else(|| user_message.to_string()))
            .await?;

        for msg in &result.messages {
            if let AgentMessage::Llm(LlmMessage::Assistant(a)) = msg {
                let text = ContentBlock::extract_text(&a.content);
                if !text.is_empty() {
                    println!("[{current}] {text}");
                }
            }
        }

        match result.transfer_signal {
            Some(signal) => {
                current = signal.target_agent().to_string();
                // Pass the original user message to the specialist.
                prompt = Some(user_message.to_string());
            }
            None => break,
        }
    }

    Ok(())
}
