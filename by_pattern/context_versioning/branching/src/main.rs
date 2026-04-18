//! Example 8: Context versioning with compaction snapshots.
//!
//! Demonstrates:
//! - `InMemoryVersionStore` — stores evicted context windows as versioned snapshots
//! - `VersioningTransformer` — wraps a `SlidingWindowTransformer` and captures
//!   every compaction event as a `ContextVersion`
//! - A simple `ContextSummarizer` implementation that labels each compacted block
//! - Querying the version store after all prompts to list recorded versions
//!
//! # Why context versioning?
//!
//! In long conversations the sliding window discards old messages to stay within
//! the token budget. Without versioning that history is lost permanently.
//! `VersioningTransformer` intercepts every compaction and saves the dropped
//! messages to a `ContextVersionStore` so you can:
//!   - Recall earlier context via RAG / semantic search
//!   - Branch or revert to a past snapshot
//!   - Audit what was compressed and when
//!
//! # Run
//!
//! ```text
//! cargo run -p context-versioning-branching
//! ```
//!
//! # Requires
//!
//! - `ANTHROPIC_API_KEY`

use std::sync::Arc;

use swink_agent::{
    Agent, AgentMessage, AgentOptions, ContentBlock, ContextSummarizer, ContextVersionStore,
    InMemoryVersionStore, LlmMessage, ModelConnections, SlidingWindowTransformer,
    VersioningTransformer,
};
use swink_agent_adapters::build_remote_connection_for_model;

// ─── SimpleSummarizer ────────────────────────────────────────────────────────

/// A trivial `ContextSummarizer` that labels each compacted block with its
/// message count. A real implementation would call a cheap LLM to produce a
/// prose summary of the evicted messages.
struct SimpleSummarizer;

impl ContextSummarizer for SimpleSummarizer {
    fn summarize(&self, messages: &[LlmMessage]) -> Option<String> {
        Some(format!(
            "[Compacted context: {} message(s) archived at this version]",
            messages.len()
        ))
    }
}

// ─── Main ─────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // ── Build versioning transformer ─────────────────────────────────────────
    //
    // The version store is kept behind an Arc so we can query it after the
    // agent finishes without the transformer owning it exclusively.
    // The Arc is typed as `dyn ContextVersionStore` so it can be passed to
    // `VersioningTransformer::new` (which expects `Arc<dyn ContextVersionStore>`)
    // and still be usable after the move via `Arc::clone`.
    let store: Arc<dyn ContextVersionStore> = Arc::new(InMemoryVersionStore::new());

    // Inner transformer: compact when context exceeds 100 k tokens, keeping
    // 50 k of tail and always preserving the 2 oldest (anchor) messages.
    let inner = SlidingWindowTransformer::new(100_000, 50_000, 2);

    // Wrap with versioning. Each compaction writes a `ContextVersion` to the
    // store; our `SimpleSummarizer` appends a human-readable label.
    let versioning = VersioningTransformer::new(inner, Arc::clone(&store))
        .with_summarizer(Arc::new(SimpleSummarizer));

    // ── Build agent ──────────────────────────────────────────────────────────
    let connection = build_remote_connection_for_model("claude-haiku-4-5-20251001")?;
    let connections = ModelConnections::new(connection, vec![]);

    let options = AgentOptions::from_connections(
        "You are a knowledgeable assistant. Give clear, concise answers.",
        connections,
    )
    // Replace the default SlidingWindowTransformer with our versioning wrapper.
    .with_transform_context(versioning);

    let mut agent = Agent::new(options);

    // ── Run prompts that build up context ────────────────────────────────────
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

    // ── Inspect recorded context versions ────────────────────────────────────
    //
    // `list_versions()` returns metadata only (no messages). Load the full
    // version with `load_version(n)` when you need the archived messages.
    let versions = store.list_versions();
    println!("--- Context version report ---");
    println!("Recorded {} context version(s)", versions.len());

    for meta in &versions {
        println!(
            "  v{}: turn={}, messages={}, has_summary={}, ts={}",
            meta.version, meta.turn, meta.message_count, meta.has_summary, meta.timestamp,
        );

        // Load and print the summary for each version.
        if let Some(version) = store.load_version(meta.version) {
            if let Some(summary) = &version.summary {
                println!("    summary: {summary}");
            }
        }
    }

    if versions.is_empty() {
        println!("  (no compaction occurred — context stayed within the token budget)");
    }

    Ok(())
}
