//! TUI with ApprovalMode::Bypassed: every bash and file-tool call is
//! auto-approved. Avoid sensitive machines. launch() applies the mode to
//! both agent dispatch and TUI state.
//! Run: cargo run
//! Requires: ANTHROPIC_API_KEY in the environment or .env

use swink_agent::{AgentOptions, ModelConnections};
use swink_agent_adapters::build_remote_connection_for_model;
use swink_agent_tui::{ApprovalMode, TuiConfig, launch, restore_terminal, setup_terminal};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let connection = build_remote_connection_for_model("claude-sonnet-4-6")?;
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
