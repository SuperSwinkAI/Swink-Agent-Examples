//! Example 7: Persistent chat with JSONL session storage.
//!
//! Demonstrates:
//! - Creating or resuming a JSONL session via `SWINK_SESSION_ID` env var
//! - An inline `CheckpointStore` backed by `Mutex<Option<Checkpoint>>`
//! - `SlidingWindowTransformer` for context management
//! - `SteeringMode::All` — drain all steering messages at once
//! - Plan-mode addendum via `DEFAULT_PLAN_MODE_ADDENDUM`
//! - Initial `SessionState` entry (`"example"` → `"persistent_chat"`)
//! - Three sequential prompts with an `Agent::steer` call after the second
//! - `AgentEvent::StateChanged` delta key reporting
//! - Saving the final transcript to the JSONL store
//! - Printing the session ID so the user can resume on the next run
//!
//! # Run
//!
//! ```text
//! cargo run -p persistent-chat-basic
//! # Resume the same session:
//! SWINK_SESSION_ID=<id> cargo run -p persistent-chat-basic
//! ```
//!
//! # Requires
//!
//! - `ANTHROPIC_API_KEY`
//! - Optional: `SWINK_SESSION_ID`

use std::io;
use std::sync::Mutex;

use swink_agent::{
    Agent, AgentEvent, AgentMessage, AgentOptions, Checkpoint, CheckpointFuture, CheckpointStore,
    ContentBlock, LlmMessage, ModelConnections, SlidingWindowTransformer, SteeringMode,
    UserMessage, DEFAULT_PLAN_MODE_ADDENDUM,
};
use swink_agent_adapters::build_remote_connection_for_model;
use swink_agent_memory::{JsonlSessionStore, SessionMeta, SessionStore, now_utc};

// ─── Inline CheckpointStore ──────────────────────────────────────────────────

/// Minimal in-memory checkpoint store.
///
/// Holds at most one checkpoint at a time. Sufficient for this example —
/// production code would use a file- or database-backed implementation.
struct InlineCheckpointStore {
    checkpoint: Mutex<Option<Checkpoint>>,
}

impl InlineCheckpointStore {
    fn new() -> Self {
        Self {
            checkpoint: Mutex::new(None),
        }
    }
}

impl CheckpointStore for InlineCheckpointStore {
    fn save_checkpoint(&self, checkpoint: Checkpoint) -> CheckpointFuture<'_, ()> {
        Box::pin(async move {
            *self
                .checkpoint
                .lock()
                .map_err(|e| io::Error::other(e.to_string()))? = Some(checkpoint);
            Ok(())
        })
    }

    fn load_checkpoint(&self, id: &str) -> CheckpointFuture<'_, Option<Checkpoint>> {
        let id = id.to_string();
        Box::pin(async move {
            let guard = self
                .checkpoint
                .lock()
                .map_err(|e| io::Error::other(e.to_string()))?;
            Ok(guard.as_ref().filter(|cp| cp.id == id).cloned())
        })
    }

    fn list_checkpoints(&self) -> CheckpointFuture<'_, Vec<String>> {
        Box::pin(async move {
            let guard = self
                .checkpoint
                .lock()
                .map_err(|e| io::Error::other(e.to_string()))?;
            Ok(guard.iter().map(|cp| cp.id.clone()).collect())
        })
    }

    fn delete_checkpoint(&self, id: &str) -> CheckpointFuture<'_, ()> {
        let id = id.to_string();
        Box::pin(async move {
            let mut guard = self
                .checkpoint
                .lock()
                .map_err(|e| io::Error::other(e.to_string()))?;
            if guard.as_ref().is_some_and(|cp| cp.id == id) {
                *guard = None;
            }
            Ok(())
        })
    }
}

// ─── Main ─────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // ── Session setup ────────────────────────────────────────────────────────
    // Resume an existing session via env var, or create a new one.
    let session_id = std::env::var("SWINK_SESSION_ID")
        .unwrap_or_else(|_| JsonlSessionStore::new_session_id());

    let sessions_dir = JsonlSessionStore::default_dir()
        .ok_or("could not determine config directory")?;
    let store = JsonlSessionStore::new(sessions_dir)?;

    // Load previous messages if this session already exists.
    let (session_meta, prior_messages) = match store.load(&session_id, None) {
        Ok(pair) => {
            println!("Resuming session: {session_id}");
            pair
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            println!("Starting new session: {session_id}");
            let meta = SessionMeta {
                id: session_id.clone(),
                title: "Persistent chat example".into(),
                created_at: now_utc(),
                updated_at: now_utc(),
                version: 1,
                sequence: 0,
            };
            (meta, Vec::new())
        }
        Err(e) => return Err(e.into()),
    };

    // ── Agent setup ──────────────────────────────────────────────────────────
    let connection = build_remote_connection_for_model("claude-haiku-4-5-20251001")?;
    let connections = ModelConnections::new(connection, vec![]);

    let options = AgentOptions::from_connections(
        "You are a helpful assistant participating in a persistent multi-turn conversation.",
        connections,
    )
    // SlidingWindowTransformer keeps the context within budget.
    .with_transform_context(SlidingWindowTransformer::new(100_000, 50_000, 2))
    // Drain all steering messages in a single pass.
    .with_steering_mode(SteeringMode::All)
    // Append the default plan-mode addendum.
    .with_plan_mode_addendum(DEFAULT_PLAN_MODE_ADDENDUM)
    // Pre-seed session state to identify this example run.
    .with_state_entry("example", "persistent_chat")
    // Attach the inline checkpoint store.
    .with_checkpoint_store(InlineCheckpointStore::new())
    // Watch for state changes and steering.
    .with_event_forwarder(|event| {
        if let AgentEvent::StateChanged { delta } = event {
            if !delta.is_empty() {
                let keys: Vec<&str> = delta.changes.keys().map(String::as_str).collect();
                println!("[state-changed] keys: {keys:?}");
            }
        }
    });

    let mut agent = Agent::new(options);

    // Restore prior conversation history if we have one.
    if !prior_messages.is_empty() {
        agent.append_messages(prior_messages);
    }

    // ── Prompts ──────────────────────────────────────────────────────────────
    let prompts = [
        "What is the difference between Rust's ownership model and garbage collection?",
        "Can you give a brief code example showing ownership transfer?",
        "Summarise what we've discussed so far in two sentences.",
    ];

    for (i, prompt) in prompts.iter().enumerate() {
        println!("\n>>> {prompt}");

        // After the second prompt, steer the agent with an additional directive.
        if i == 2 {
            let steering_msg = AgentMessage::Llm(LlmMessage::User(UserMessage {
                content: vec![ContentBlock::Text {
                    text: "Please keep your summary under 50 words.".to_string(),
                }],
                timestamp: 0,
                cache_hint: None,
            }));
            agent.steer(steering_msg);
            println!("[steering] injected: keep summary under 50 words");
        }

        let result = agent.prompt_text(*prompt).await?;

        for msg in &result.messages {
            if let AgentMessage::Llm(LlmMessage::Assistant(a)) = msg {
                println!("{}", ContentBlock::extract_text(&a.content));
            }
        }
    }

    // ── Persist transcript ───────────────────────────────────────────────────
    // `save` takes `&[AgentMessage]`, so we borrow directly from agent state.
    store.save(&session_id, &session_meta, &agent.state().messages)?;
    println!("\nSession saved. Resume with:");
    println!("  SWINK_SESSION_ID={session_id} cargo run -p persistent-chat-basic");

    Ok(())
}
