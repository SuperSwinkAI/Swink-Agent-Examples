//! Example: MCP client connected to Context7 for live library documentation.
//!
//! Demonstrates swink-agent-mcp (HTTP/SSE transport, bearer-token auth,
//! tool_prefix), ToolDenyListPolicy, and AuditLogger.

use swink_agent::prelude::*;
use swink_agent_adapters::build_remote_connection_for_model;
use swink_agent_mcp::{McpManager, McpServerConfig, McpTransport};
use swink_agent_policies::{AuditLogger, JsonlAuditSink, ToolDenyListPolicy};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // ── 1. Read API keys ─────────────────────────────────────────────────────

    let context7_token = std::env::var("CONTEXT7_API_KEY")
        .map_err(|_| "CONTEXT7_API_KEY not set")?;

    // ── 2. Connect to Context7 via MCP/SSE ──────────────────────────────────

    let mut mcp = McpManager::new(vec![McpServerConfig {
        name: "context7".into(),
        transport: McpTransport::Sse {
            url: "https://mcp.context7.com/mcp".into(),
            bearer_token: Some(context7_token),
            bearer_auth: None,
            headers: Default::default(),
        },
        tool_prefix: Some("ctx7".into()),
        tool_filter: None,
        requires_approval: false,
        connect_timeout_ms: None,
        discovery_timeout_ms: None,
    }]);
    mcp.connect_all().await?;
    let mcp_tools = mcp.tools();

    println!("Connected to Context7 — {} tool(s) discovered", mcp_tools.len());

    // ── 3. Build model connection ────────────────────────────────────────────

    let connection = build_remote_connection_for_model("claude-haiku-4-5-20251001")?;
    let connections = ModelConnections::new(connection, vec![]);

    // ── 4. Policies ──────────────────────────────────────────────────────────

    // Deny a nonexistent tool to demonstrate the deny-list pattern without
    // actually blocking any real Context7 tools.
    let deny_list = ToolDenyListPolicy::new(["ctx7_nonexistent_tool"]);

    let audit_path = "/tmp/context7-audit.jsonl";
    let audit_logger = AuditLogger::new(JsonlAuditSink::new(audit_path));

    // ── 5. Build agent options ───────────────────────────────────────────────

    let system_prompt = "You are a documentation assistant. \
        Use the ctx7 tools to look up current library documentation. \
        First resolve the library ID, then query the docs.";

    let options = AgentOptions::from_connections(system_prompt, connections)
        .with_tools(mcp_tools)
        .with_approval_mode(ApprovalMode::Bypassed)
        .with_pre_dispatch_policy(deny_list)
        .with_post_turn_policy(audit_logger);

    // ── 6. Run ──────────────────────────────────────────────────────────────

    let prompt = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "How do I define routes in Axum 0.7?".to_string());

    println!(">>> {prompt}");
    let mut agent = Agent::new(options);
    let result = agent.prompt_text(&prompt).await?;
    println!("Assistant: {}", result.assistant_text());

    println!("\nAudit log written to: {audit_path}");

    Ok(())
}
