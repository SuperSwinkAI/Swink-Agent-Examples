# ~~Plan~~: by_pattern/tui/yolo — IMPLEMENTED

~~Blocked on SuperSwinkAI/Swink-Agent#565.~~ Fixed in 0.7.2 (#567).

## What this example will be

A TUI agent with all built-in tools and `ApprovalMode::Bypassed` — every tool
call (bash, read, write) executes immediately with no approval prompt.

## Why it's blocked

Setting `ApprovalMode::Bypassed` currently requires updating two independent
pieces of state: `AgentOptions::with_approval_mode()` (controls the agent
dispatch loop) and `app.approval_mode` (controls the TUI overlay). There is no
single API that keeps both in sync, and `launch()` has no way to pass an
approval mode at all.

Once #565 is resolved this example becomes straightforward — likely a
one-liner change on top of `by_pattern/tui/basic`.

## Intended implementation (post-fix)

```rust
use swink_agent::{AgentOptions, ApprovalMode, ModelConnections};
use swink_agent_adapters::build_remote_connection_for_model;
use swink_agent_tui::{TuiConfig, launch, restore_terminal, setup_terminal};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let connection = build_remote_connection_for_model("claude-haiku-4-5-20251001")?;
    let connections = ModelConnections::new(connection, vec![]);

    let options = AgentOptions::from_connections(
        "You are a capable, autonomous assistant with access to shell and file tools. \
         Execute tasks directly and completely without asking for confirmation.",
        connections,
    )
    .with_default_tools();

    let mut terminal = setup_terminal()?;
    // approval_mode param added by #565 fix
    let result = launch(TuiConfig::load(), ApprovalMode::Bypassed, &mut terminal, options).await;
    restore_terminal()?;

    result
}
```
