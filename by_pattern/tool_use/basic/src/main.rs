//! Example: register tools with an Agent and set up the approval callback.
//!
//! Demonstrates how to wire up `BashTool`, `ReadFileTool`, `WriteFileTool`,
//! and `EditFileTool`, configure an approval callback via
//! `with_approve_tool_async`, and run prompts that actually invoke tools.
//!
//! Also shows how to create a custom tool with [`FnTool`] using closures
//! instead of implementing the [`AgentTool`] trait manually.

use schemars::JsonSchema;
use serde::Deserialize;
use swink_agent::ToolApproval;
use swink_agent::prelude::*;
use swink_agent_adapters::build_remote_connection_for_model;

// ─── Custom tool params ─────────────────────────────────────────────────────

/// Parameters for the `get_weather` custom tool.
#[derive(Deserialize, JsonSchema)]
struct GetWeatherParams {
    /// The city to look up weather for (e.g. "Austin").
    city: String,
}

// ─── Main ───────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // Step 1a: Create built-in tools.
    // `builtin_tools()` returns Vec<Arc<dyn AgentTool>> with BashTool, ReadFileTool,
    // WriteFileTool, and EditFileTool (added in 0.7.3).
    // EditFileTool applies surgical find-and-replace edits atomically and supports
    // stale-read detection via an optional SHA-256 hash of the file as last read.
    let mut tools = builtin_tools();

    // Step 1b: Create a custom tool using `FnTool` — no need to implement `AgentTool` manually.
    // `with_execute_typed::<T, _, _>` derives the schema from `T` and deserializes
    // incoming JSON into `T` before the closure runs. Invalid JSON returns
    // `AgentToolResult::error("invalid parameters: ...")` automatically — no
    // `with_schema_for` or manual `params["…"]` lookups needed.
    let weather = FnTool::new(
        "get_weather",
        "Weather",
        "Get the current weather for a city.",
    )
    .with_execute_typed::<GetWeatherParams, _, _>(|params, _cancel| async move {
        // In a real application this would call a weather API.
        AgentToolResult::text(format!("72°F and sunny in {}", params.city))
    })
    .into_tool();

    tools.push(weather);

    // Step 2: Connect to a real model so tools are actually invoked.
    let connection = build_remote_connection_for_model("claude-haiku-4-5-20251001")?;
    let connections = ModelConnections::new(connection, vec![]);

    // Step 3: Build options with tools and an approval callback.
    //
    // `with_approve_tool_async` registers an async approval callback that is
    // invoked for every tool call. It avoids the verbose `Pin<Box<dyn Future<...>>>`
    // ceremony that `with_approve_tool` requires.
    //
    // To only prompt for tools that declare `requires_approval() == true`
    // (e.g. BashTool and WriteFileTool, but not ReadFileTool), wrap the
    // callback with `selective_approve` instead:
    //
    //   .with_approve_tool(selective_approve(|req| Box::pin(async move { ... })))
    let options = AgentOptions::from_connections(
        "You are a helpful coding assistant with access to shell and file tools. \
         Use edit_file for targeted changes to existing files — it is safer than \
         overwriting the whole file with write_file.",
        connections,
    )
    .with_tools(tools)
    .with_approve_tool_async(|req| async move {
        println!(
            "  [approval] tool='{}' args={}",
            req.tool_name, req.arguments
        );
        ToolApproval::Approved
    });

    // Step 4: Create the agent and run prompts that actually invoke tools.
    let mut agent = Agent::new(options);

    for prompt in [
        "Use bash to count how many .toml files are in the current directory.",
        "What's the weather like in Austin right now?",
    ] {
        println!(">>> {prompt}");
        let result = agent.prompt_text(prompt).await?;
        println!("Assistant: {}\n", result.assistant_text());
    }

    Ok(())
}
