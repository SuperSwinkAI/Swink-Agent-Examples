//! Example: single-agent web research with file output.
//!
//! Pass a research query on the command line. The agent searches the web,
//! fetches relevant pages, synthesizes findings into a Markdown report, and
//! writes it to `~/research-<slug>.md`.
//!
//! # Run
//!
//! ```text
//! cargo run -p research-basic -- "history of the Rust programming language"
//! ```
//!
//! # Requires
//!
//! - `ANTHROPIC_API_KEY` in environment or `.env` file

use dotenvy::dotenv;
use swink_agent::{
    Agent, AgentEvent, AgentOptions, ApprovalMode, ModelConnections, Plugin, builtin_tools,
};
use swink_agent_adapters::build_remote_connection_for_model;
use swink_agent_plugin_web::WebPlugin;

const MODEL: &str = "claude-sonnet-4-6";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let query = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: cargo run -p research-basic -- \"your research query\"");
        std::process::exit(1);
    });

    // Build a filesystem-safe slug for the output filename.
    let slug = query
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let report_path = format!("{home}/research-{slug}.md");

    // Extract web tools directly to avoid the `web.` namespace prefix that
    // `with_plugin()` adds — Anthropic's API rejects dots in tool names.
    let web_plugin = WebPlugin::new()?;
    let mut tools = builtin_tools();
    tools.extend(web_plugin.tools());

    let connection = build_remote_connection_for_model(MODEL)?;
    let options = AgentOptions::from_connections(
        "You are a research assistant. When given a topic:\n\
         1. Use search to find relevant sources (2–4 searches).\n\
         2. Use fetch on the most promising URLs to read their content.\n\
         3. Synthesize findings into a well-structured Markdown report with:\n\
            - An executive summary\n\
            - Key findings (with source citations as inline links)\n\
            - A conclusion\n\
         4. Write the COMPLETE report to the file path given in the prompt \
            using write_file. Do not truncate or summarise — write everything.",
        ModelConnections::new(connection, vec![]),
    )
    .with_tools(tools)
    .with_event_forwarder(|event| match event {
        AgentEvent::BeforeLlmCall { messages, .. } => {
            println!("  [thinking] ({} messages so far)...", messages.len());
        }
        AgentEvent::ToolExecutionStart { name, .. } => {
            println!("  [tool] {name}");
        }
        AgentEvent::ToolExecutionEnd { name, is_error, result, .. } => {
            if is_error {
                let msg = swink_agent::ContentBlock::extract_text(&result.content);
                println!("  [tool] {name} — error: {msg}");
            }
        }
        _ => {}
    })
    .with_approval_mode(ApprovalMode::Bypassed);

    let mut agent = Agent::new(options);

    println!("Query : {query}");
    println!("Output: {report_path}\n");

    let result = agent
        .prompt_text(format!(
            "Research the following topic and write the complete Markdown report \
             to '{report_path}':\n\n{query}"
        ))
        .await?;

    println!("{}", result.assistant_text());
    Ok(())
}
