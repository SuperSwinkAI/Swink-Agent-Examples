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
    ContentBlock, DEFAULT_PLAN_MODE_ADDENDUM, LlmMessage, ModelConnections,
    SlidingWindowTransformer, SteeringMode, UserMessage,
};
use swink_agent_adapters::build_remote_connection_for_model;
use swink_agent_memory::{JsonlSessionStore, SessionMeta, SessionStore, now_utc};

/// In-memory store holding at most one checkpoint.
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let session_id =
        std::env::var("SWINK_SESSION_ID").unwrap_or_else(|_| JsonlSessionStore::new_session_id());

    let sessions_dir =
        JsonlSessionStore::default_dir().ok_or("could not determine config directory")?;
    let store = JsonlSessionStore::new(sessions_dir)?;

    let (session_meta, prior_messages) = match store.load(&session_id, None) {
        Ok(pair) => {
            println!("Resuming session: {session_id}");
            pair
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            println!("Starting new session: {session_id}");
            let meta = SessionMeta::new(
                session_id.clone(),
                "Persistent chat example",
                now_utc(),
                now_utc(),
            );
            (meta, Vec::new())
        }
        Err(e) => return Err(e.into()),
    };

    let connection = build_remote_connection_for_model("claude-haiku-4-5-20251001")?;
    let connections = ModelConnections::new(connection, vec![]);

    let options = AgentOptions::from_connections(
        "You are a helpful assistant participating in a persistent multi-turn conversation.",
        connections,
    )
    .with_transform_context(SlidingWindowTransformer::new(100_000, 50_000, 2))
    .with_steering_mode(SteeringMode::All)
    .with_plan_mode_addendum(DEFAULT_PLAN_MODE_ADDENDUM)
    .with_state_entry("example", "persistent_chat")
    .with_checkpoint_store(InlineCheckpointStore::new())
    .with_event_forwarder(|event| {
        if let AgentEvent::StateChanged { delta } = event
            && !delta.is_empty()
        {
            let keys: Vec<&str> = delta.changes.keys().map(String::as_str).collect();
            println!("[state-changed] keys: {keys:?}");
        }
    });

    let mut agent = Agent::new(options);

    if !prior_messages.is_empty() {
        agent.append_messages(prior_messages);
    }

    let prompts = [
        "What is the difference between Rust's ownership model and garbage collection?",
        "Can you give a brief code example showing ownership transfer?",
        "Summarise what we've discussed so far in two sentences.",
    ];

    for (i, prompt) in prompts.iter().enumerate() {
        println!("\n>>> {prompt}");

        if i == 2 {
            let steering_msg = AgentMessage::Llm(LlmMessage::User(
                UserMessage::new(vec![ContentBlock::Text {
                    text: "Please keep your summary under 50 words.".to_string(),
                }])
                .with_timestamp(0),
            ));
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

    store.save(&session_id, &session_meta, &agent.state().messages)?;
    println!("\nSession saved. Resume with:");
    println!("  SWINK_SESSION_ID={session_id} cargo run -p persistent-chat-basic");

    Ok(())
}
