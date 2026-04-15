# tool_use/basic

Demonstrates the tool system: registering built-in tools, creating a custom tool with `FnTool`, and wiring up an async approval callback.

## What it shows

- `builtin_tools()` — returns `[BashTool, ReadFileTool, WriteFileTool]`
- `FnTool` builder — `new` → `with_schema_for::<T>()` → `with_execute_simple` → `into_tool()`
- `AgentOptions::with_tools` + `with_approve_tool_async`
- `ToolApproval::Approved` / `Denied` verdicts

## Run

```bash
cargo run
```

No API key required — uses a mock `StreamFn`.

## Swapping in a real provider

```toml
swink-agent-adapters = { version = "0.5", features = ["anthropic"] }
```

```rust
use swink_agent_adapters::{build_remote_connection, remote_preset_keys};
let connection = build_remote_connection(remote_preset_keys::anthropic::HAIKU_45)?;
let connections = ModelConnections::new(connection, vec![]);
let options = AgentOptions::from_connections(system_prompt, connections)
    .with_tools(tools)
    .with_approve_tool_async(|req| async move { ToolApproval::Approved });
```
