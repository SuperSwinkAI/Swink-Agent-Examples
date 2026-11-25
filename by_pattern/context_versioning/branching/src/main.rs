//! Context versioning with InMemoryVersionStore and VersioningTransformer.
//! Records compacted context snapshots and lists their versions.
//! Run: cargo run -p context-versioning-branching
//! Requires: ANTHROPIC_API_KEY

use std::sync::Arc;

use swink_agent::{
    Agent, AgentMessage, AgentOptions, ContentBlock, ContextSummarizer, ContextVersionStore,
    InMemoryVersionStore, LlmMessage, ModelConnections, SlidingWindowTransformer,
    VersioningTransformer,
};
use swink_agent_adapters::build_remote_connection_for_model;

/// Label each compacted context block with its message count.
struct SimpleSummarizer;

impl ContextSummarizer for SimpleSummarizer {
    fn summarize(&self, messages: &[LlmMessage]) -> Option<String> {
        Some(format!(
            "[Compacted context: {} message(s) archived at this version]",
            messages.len()
        ))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let store: Arc<dyn ContextVersionStore> = Arc::new(InMemoryVersionStore::new());

    let inner = SlidingWindowTransformer::new(100_000, 50_000, 2);

    let versioning = VersioningTransformer::new(inner, Arc::clone(&store))
        .with_summarizer(Arc::new(SimpleSummarizer));

    let connection = build_remote_connection_for_model("claude-haiku-4-5-20251001")?;
    let connections = ModelConnections::new(connection, vec![]);

    let options = AgentOptions::from_connections(
        "You are a knowledgeable assistant. Give clear, concise answers.",
        connections,
    )
    .with_transform_context(versioning);

    let mut agent = Agent::new(options);

    let prompts = [
        "What is Rust?",
        "What is tokio?",
        "How do Rust and tokio relate to each other?",
        "Give me a short code example that combines both.",
        "Summarise everything we have discussed in three sentences.",
    ];

    for prompt in &prompts {
        println!(">>> {prompt}");
        let result = agent.prompt_text(*prompt).await?;
        for msg in &result.messages {
            if let AgentMessage::Llm(LlmMessage::Assistant(a)) = msg {
                println!("{}\n", ContentBlock::extract_text(&a.content));
            }
        }
    }

    let versions = store.list_versions();
    println!("--- Context version report ---");
    println!("Recorded {} context version(s)", versions.len());

    for meta in &versions {
        println!(
            "  v{}: turn={}, messages={}, has_summary={}, ts={}",
            meta.version, meta.turn, meta.message_count, meta.has_summary, meta.timestamp,
        );

        if let Some(version) = store.load_version(meta.version)
            && let Some(summary) = &version.summary
        {
            println!("    summary: {summary}");
        }
    }

    if versions.is_empty() {
        println!("  (no compaction occurred — context stayed within the token budget)");
    }

    Ok(())
}
