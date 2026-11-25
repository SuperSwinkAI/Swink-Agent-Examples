//! Register built-in tools, configure with_approve_tool_async, and invoke tools.
//! Also demonstrates a custom FnTool backed by closures.

use schemars::JsonSchema;
use serde::Deserialize;
use swink_agent::ToolApproval;
use swink_agent::prelude::*;
use swink_agent_adapters::build_remote_connection_for_model;

/// Parameters for the `get_weather` custom tool.
#[derive(Deserialize, JsonSchema)]
struct GetWeatherParams {
    /// The city to look up weather for (e.g. "Austin").
    city: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let mut tools = builtin_tools();

    let weather = FnTool::new(
        "get_weather",
        "Weather",
        "Get the current weather for a city.",
    )
    .with_execute_typed::<GetWeatherParams, _, _>(|params, _cancel| async move {
        AgentToolResult::text(format!("72°F and sunny in {}", params.city))
    })
    .into_tool();

    tools.push(weather);

    let connection = build_remote_connection_for_model("claude-haiku-4-5-20251001")?;
    let connections = ModelConnections::new(connection, vec![]);

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
