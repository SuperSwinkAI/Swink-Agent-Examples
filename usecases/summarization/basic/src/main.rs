use swink_agent::{Agent, AgentMessage, AgentOptions, ContentBlock, LlmMessage, ModelConnections};
use swink_agent_adapters::build_remote_connection_for_model;

const SAMPLE_TEXT: &str = "\
Large language models are increasingly deployed as autonomous agents that plan and execute \
multi-step tasks, yet they exhibit a well-documented failure mode: they confidently produce \
plausible-sounding but factually incorrect outputs — a phenomenon researchers call hallucination. \
Unlike retrieval-augmented systems that ground responses in retrieved documents, pure generation \
models have no mechanism to distinguish between remembered training data and confabulation. \
Recent work on tool-use and verification loops attempts to close this gap by giving models \
access to search, code execution, and external APIs, forcing claims to be checked rather than \
assumed. Critics argue that this merely shifts the problem: models can now hallucinate tool \
invocations or misinterpret tool results, introducing new failure modes at the integration \
layer. The open question is whether grounding alone is sufficient for safety-critical \
applications, or whether a fundamentally different approach to confidence calibration is needed.";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let connection = build_remote_connection_for_model("claude-haiku-4-5-20251001")?;
    let connections = ModelConnections::new(connection, vec![]);
    let options = AgentOptions::from_connections(
        "You are a summarization assistant. Provide concise, one-paragraph summaries.",
        connections,
    );

    let mut agent = Agent::new(options);

    let prompts = [
        format!("Read the following passage and identify the top 3 technical claims. For each, note whether it's broadly accepted, actively debated, or contested:\n\n{SAMPLE_TEXT}"),
        "Now distill the core tension in that passage into a single sentence a non-technical engineering manager could read in a board meeting.".to_string(),
    ];

    for prompt in &prompts {
        println!(">>> {prompt}\n");
        let result = agent.prompt_text(prompt).await?;
        for msg in &result.messages {
            if let AgentMessage::Llm(LlmMessage::Assistant(a)) = msg {
                println!("{}\n", ContentBlock::extract_text(&a.content));
            }
        }
    }
    Ok(())
}
