//! Example: YOLO agent — TUI with all tools and zero approval prompts.
//!
//! "You Only Live Once" — every tool call (bash, file read, file write) is
//! auto-approved. The agent never pauses to ask permission. Use this when you
//! trust the model and want maximum autonomy; avoid it on sensitive machines.
//!
//! `ApprovalMode::Bypassed` on `AgentOptions` is the single source of truth in
//! 0.7.2+: `launch()` propagates it to both the agent dispatch loop and the
//! TUI state, so no manual sync is needed.
//!
//! # Run
//!
//! ```text
//! cargo run
//! ```
//!
//! # Requires
//!
//! - `ANTHROPIC_API_KEY` in environment or `.env` file

use swink_agent::{AgentOptions, ModelConnections};
use swink_agent_adapters::build_remote_connection_for_model;
use swink_agent_tui::{ApprovalMode, TuiConfig, launch, restore_terminal, setup_terminal};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let connection = build_remote_connection_for_model("claude-haiku-4-5-20251001")?;
    let connections = ModelConnections::new(connection, vec![]);

    let options = AgentOptions::from_connections(
        "You are a capable, autonomous assistant with access to shell and file tools. \
         Execute tasks directly and completely without asking for confirmation. \
         Prefer doing over asking.",
        connections,
    )
    .with_default_tools()
    .with_approval_mode(ApprovalMode::Bypassed);

    let mut terminal = setup_terminal()?;
    let result = launch(TuiConfig::load(), &mut terminal, options).await;
    restore_terminal()?;

    result
}
