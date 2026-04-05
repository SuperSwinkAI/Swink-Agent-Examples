use swink_agent::{Agent, AgentMessage, AgentOptions, ContentBlock, LlmMessage, ModelConnections};
use swink_agent_adapters::{build_remote_connection, remote_preset_keys};

const SAMPLE_TEXT: &str = "\
Rust is a multi-paradigm, general-purpose programming language that emphasizes performance, \
type safety, and concurrency. It enforces memory safety without a garbage collector by using \
a system of ownership with a set of rules that the compiler checks at compile time. Rust was \
originally designed by Graydon Hoare at Mozilla Research, with contributions from Dave Herman, \
Brendan Eich, and others. The designers refined the language while writing the Servo \
experimental browser engine and the Rust compiler itself. Rust has been adopted by major \
technology companies for systems programming, and it has topped Stack Overflow's annual \
survey as the most admired programming language every year since 2016.";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let connection = build_remote_connection(remote_preset_keys::anthropic::HAIKU_45)?;
    let connections = ModelConnections::new(connection, vec![]);
    let options = AgentOptions::from_connections(
        "You are a summarization assistant. Provide concise, one-paragraph summaries.",
        connections,
    );

    let mut agent = Agent::new(options);
    let prompt = format!("Summarize the following text in 2-3 sentences:\n\n{SAMPLE_TEXT}");
    let result = agent.prompt_text(&prompt).await?;

    for msg in &result.messages {
        if let AgentMessage::Llm(LlmMessage::Assistant(a)) = msg {
            println!("{}", ContentBlock::extract_text(&a.content));
        }
    }
    Ok(())
}
