//! Example 9: Hot-reload script tools with a ToolWatcher.
//!
//! Demonstrates:
//! - Loading shell-command tools from TOML files at runtime
//! - `ToolWatcher` monitoring a directory and streaming tool updates
//! - `SandboxPolicy` restricting file access
//! - `ApprovalMode::Bypassed` for non-interactive demos
//! - Hot-reload: modifying a tool definition at runtime and receiving an update
//!
//! # Run
//!
//! ```text
//! cargo run -p hot-reload-scripting-agent
//! ```
//!
//! # Requires
//!
//! - `ANTHROPIC_API_KEY`

use std::time::Duration;

use swink_agent::hot_reload::ToolWatcher;
use swink_agent::ApprovalMode;
use swink_agent::prelude::*;
use swink_agent_adapters::build_remote_connection_for_model;
use swink_agent_policies::SandboxPolicy;
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // Step 1: Start the ToolWatcher on ./tools.
    let ct = CancellationToken::new();
    let tools_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tools");
    println!("[watcher] Watching: {}", tools_dir.display());

    let watcher = ToolWatcher::new(&tools_dir)
        .map_err(|e| format!("ToolWatcher::new failed: {e}"))?;
    let mut update_rx = watcher.start(ct.clone()).await;

    // Step 2: Wait briefly for the initial scan.
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Step 3: Receive the initial tool list.
    let initial_tools = update_rx
        .recv()
        .await
        .ok_or("ToolWatcher did not emit an initial tool list")?;
    println!(
        "[watcher] Initial scan: {} tool(s) loaded",
        initial_tools.len()
    );
    for tool in &initial_tools {
        println!("[watcher]   - {}: {}", tool.name(), tool.description());
    }

    // Step 4: Build connection and agent.
    let connection = build_remote_connection_for_model("claude-haiku-4-5-20251001")?;
    let connections = ModelConnections::new(connection, vec![]);

    // Sandbox: restrict file access to the tools directory.
    let sandbox = SandboxPolicy::new(&tools_dir);

    let options = AgentOptions::from_connections(
        "You are a helpful assistant with access to script tools. \
         Use the available tools to complete the user's request.",
        connections,
    )
    .with_tools(initial_tools)
    .with_pre_dispatch_policy(sandbox)
    .with_approval_mode(ApprovalMode::Bypassed);

    let mut agent = Agent::new(options);

    // Step 5: Run the prompt.
    let prompt = "Greet the user named 'Alice' and then count the words in 'the quick brown fox'.";
    println!("\n>>> {prompt}");
    let result = agent.prompt_text(prompt).await?;
    println!("Assistant: {}", result.assistant_text());

    // Step 6: Demonstrate hot-reload by spawning a background task that
    //         modifies greet.toml after 1 second and then prints the update.
    let tools_dir_clone = tools_dir.clone();
    let ct_clone = ct.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(1)).await;

        // Overwrite greet.toml with a slightly different command.
        let updated_toml = r#"name = "greet"
description = "Greet a person by name (updated)"
command = "echo 'Hey there, {name}! Hot-reload worked!'"

[parameters_schema]
type = "object"
required = ["name"]

[parameters_schema.properties.name]
type = "string"
description = "The name of the person to greet"
"#;
        let greet_path = tools_dir_clone.join("greet.toml");
        if let Err(e) = std::fs::write(&greet_path, updated_toml) {
            eprintln!("[hot-reload] Failed to write updated tool: {e}");
        } else {
            println!("[hot-reload] Wrote updated greet.toml");
        }

        // Wait for ToolWatcher to detect and emit the update.
        tokio::time::sleep(Duration::from_millis(800)).await;
        ct_clone.cancel();
    });

    // Wait for the hot-reload notification.
    if let Some(updated_tools) = update_rx.recv().await {
        println!(
            "[hot-reload] Tools updated: {} tool(s)",
            updated_tools.len()
        );
        println!(
            "[hot-reload] In a real app you would re-create the agent with updated_tools."
        );
    }

    // Wait for background task to finish.
    tokio::time::sleep(Duration::from_millis(200)).await;

    Ok(())
}
