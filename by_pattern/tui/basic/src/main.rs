//! Example: interactive terminal UI powered by `swink-agent-tui`.
//!
//! Launches a full ratatui chat interface backed by Anthropic Haiku 4.5.
//! The TUI handles rendering, keyboard input, streaming output, tool approval
//! prompts, and session saving automatically.
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
//!
//! # Key bindings (built into the TUI)
//!
//! - `Enter` — send message
//! - `Ctrl+C` / `:q` — quit
//! - `Ctrl+Z` — abort running generation
//! - `Esc` — cancel/close overlay
//! - `↑ / ↓` — scroll conversation

use swink_agent::{AgentOptions, ModelConnections};
use swink_agent_adapters::build_remote_connection_for_model;
use swink_agent_tui::{TuiConfig, launch, restore_terminal, setup_terminal};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // Build a connection to Haiku 4.5. Swap the model ID here to use any other
    // remote provider — the rest of the code is identical.
    let connection = build_remote_connection_for_model("claude-haiku-4-5-20251001")?;
    let connections = ModelConnections::new(connection, vec![]);

    // `with_default_tools()` adds BashTool, ReadFileTool, and WriteFileTool.
    // Remove it (or call `.with_tools(vec![])`) for a pure chat experience.
    let options = AgentOptions::from_connections("You are a helpful assistant.", connections)
        .with_default_tools();

    // `setup_terminal` switches to raw mode + alternate screen.
    // `restore_terminal` must be called on exit — `launch` returns only after
    // the user quits, so the restore call below always runs.
    let mut terminal = setup_terminal()?;
    let result = launch(TuiConfig::load(), &mut terminal, options).await;
    restore_terminal()?;

    result
}
