# AGENTS.md — parallel_tools/fan_out_fetch

## Scope

Shows how `ToolExecutionPolicy` controls whether an agent's tool calls run sequentially, concurrently, or in priority-ordered batches.

## Key files

- `src/main.rs` — four `FnTool` definitions, `Mode` enum parsed from CLI, and `run_with_policy` helper
- `Cargo.toml` — depends on `swink-agent`, `swink-agent-adapters`, `schemars`, `serde`

## Dependencies and features

| Crate | Version | Notes |
|---|---|---|
| `swink-agent` | `0.7` | `Agent`, `AgentOptions`, `FnTool`, `ToolExecutionPolicy`, `ModelConnections` |
| `swink-agent-adapters` | `0.7` (feature `anthropic`) | `build_remote_connection_for_model` |
| `schemars` | `1` | `JsonSchema` derive for tool parameter structs |
| `serde` | `1` | `Deserialize` derive for tool parameter structs |

## Configuration surface

- `ANTHROPIC_API_KEY` — required at runtime
- `--mode sequential|concurrent|priority` — selects the dispatch policy
- Sleep duration in each tool (500 ms) — adjust to exaggerate or shrink the timing delta

## How to modify safely

- To add a tool, define a new `*Params` struct and a builder function following the existing pattern, then push the result into `tools`.
- To use a custom priority function, replace `Arc::new(|call| ...)` with any `Fn(&ToolCallSummary<'_>) -> i32 + Send + Sync`.
- `run_with_policy` creates a fresh `Agent` each call, so mode switching does not leak conversation history.

## Testing guidance

```bash
cargo check
cargo run -- --mode concurrent   # requires ANTHROPIC_API_KEY
```
