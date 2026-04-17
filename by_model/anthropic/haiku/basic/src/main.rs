use swink_agent::{Agent, AgentMessage, AgentOptions, ContentBlock, LlmMessage, ModelConnections};
use swink_agent_adapters::build_remote_connection_for_model;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let connection = build_remote_connection_for_model("claude-haiku-4-5-20251001")?;
    let connections = ModelConnections::new(connection, vec![]);
    let options = AgentOptions::from_connections("You are a helpful assistant.", connections);

    let mut agent = Agent::new(options);

    for prompt in [
        "You're claude-haiku — Anthropic's fastest, cheapest model. Be honest: what tasks do you handle noticeably worse than Sonnet or Opus, and why?",
        "Given those limitations, rank these three use cases by how well-suited you are: (1) real-time customer chat triage, (2) 80-page contract review, (3) generating SQL from a natural-language query.",
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
