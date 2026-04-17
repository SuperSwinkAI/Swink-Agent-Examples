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

use std::sync::atomic::{AtomicU32, Ordering};

use dotenvy::dotenv;
use swink_agent::{
    Agent, AgentEvent, AgentOptions, ApprovalMode, ModelConnections, Plugin, PolicyContext,
    PolicyVerdict, PostTurnPolicy, TurnPolicyContext, builtin_tools,
};
use swink_agent_adapters::build_remote_connection_for_model;
use swink_agent_plugin_web::WebPlugin;

const MAX_TURNS: u32 = 12;

struct TurnLimitPolicy {
    count: AtomicU32,
}

impl TurnLimitPolicy {
    fn new() -> Self {
        Self { count: AtomicU32::new(0) }
    }
}

impl PostTurnPolicy for TurnLimitPolicy {
    fn name(&self) -> &str { "turn_limit" }
    fn evaluate(&self, _ctx: &PolicyContext<'_>, _turn: &TurnPolicyContext<'_>) -> PolicyVerdict {
        let n = self.count.fetch_add(1, Ordering::SeqCst) + 1;
        if n >= MAX_TURNS {
            eprintln!("  [limit] max turns ({MAX_TURNS}) reached — stopping");
            PolicyVerdict::Stop(format!("turn limit {MAX_TURNS} reached"))
        } else {
            PolicyVerdict::Continue
        }
    }
}

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
    let web_plugin = WebPlugin::from_config(
        WebPlugin::builder().with_max_content_length(500_000).build(),
    )?;
    let mut tools = builtin_tools();
    // Only add search and fetch — extract and screenshot require Playwright.
    tools.extend(
        web_plugin
            .tools()
            .into_iter()
            .filter(|t| matches!(t.name(), "search" | "fetch")),
    );

    let connection = build_remote_connection_for_model(MODEL)?;
    let options = AgentOptions::from_connections(
        "You are a research assistant. Follow these steps exactly:\n\
         1. Run 2–3 web searches on the topic.\n\
         2. Fetch the 3–4 most relevant URLs. If a fetch fails, skip it — do NOT retry.\n\
         3. Using whatever content you successfully fetched, write a comprehensive \
            Markdown report with: an executive summary, key findings with source \
            citations, and a conclusion.\n\
         4. Call write_file ONCE with the complete report. Do not loop back to search \
            or fetch after this step. Writing the file is your final action.",
        ModelConnections::new(connection, vec![]),
    )
    .with_tools(tools)
    .with_post_turn_policy(TurnLimitPolicy::new())
    .with_event_forwarder(|event| match event {
        AgentEvent::BeforeLlmCall { messages, .. } => {
            println!("  [thinking] ({} messages)...", messages.len());
        }
        AgentEvent::ToolExecutionStart { name, arguments, .. } => {
            // For write_file show the path; for search show the query; others just name.
            if name == "write_file" {
                let path = arguments.get("path").and_then(|v| v.as_str()).unwrap_or("?");
                println!("  [tool] write_file → {path}");
            } else if name == "search" {
                let q = arguments.get("query").and_then(|v| v.as_str()).unwrap_or("?");
                println!("  [tool] search: {q}");
            } else if name == "fetch" {
                let url = arguments.get("url").and_then(|v| v.as_str()).unwrap_or("?");
                println!("  [tool] fetch: {url}");
            } else {
                println!("  [tool] {name}");
            }
        }
        AgentEvent::ToolExecutionEnd { name, is_error, result, .. } => {
            if is_error {
                let msg = swink_agent::ContentBlock::extract_text(&result.content);
                println!("  [tool] {name} failed: {msg}");
            } else if name == "write_file" {
                println!("  [tool] write_file ✓");
            }
        }
        AgentEvent::MessageUpdate { delta } => {
            if let swink_agent::AssistantMessageDelta::Text { delta: text, .. } = delta {
                print!("{text}");
                let _ = std::io::Write::flush(&mut std::io::stdout());
            }
        }
        AgentEvent::MessageEnd { .. } => {
            println!();
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

    if result.error.is_some() {
        eprintln!("  [error] {:?}", result.error);
    }
    Ok(())
}
