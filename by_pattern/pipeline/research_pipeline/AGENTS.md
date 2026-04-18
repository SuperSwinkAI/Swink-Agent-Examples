# AGENTS.md — pipeline/research_pipeline

## Scope

3-stage research pipeline (outline → draft → critic loop) using `swink-agent-patterns`.

## Key files

- `src/main.rs` — registers agents and pipelines, runs outline+draft sequentially, then critic in a loop
- `Cargo.toml` — depends on `swink-agent`, `swink-agent-adapters`, `swink-agent-patterns`, `tokio-util`

## Dependencies and features

| Crate | Version | Notes |
|---|---|---|
| `swink-agent` | `0.7` | `Agent`, `AgentOptions`, `ModelConnections` |
| `swink-agent-adapters` | `0.7` (feature `anthropic`) | `build_remote_connection_for_model` |
| `swink-agent-patterns` | `0.7` | `Pipeline`, `SimpleAgentFactory`, `PipelineRegistry`, `PipelineExecutor`, `ExitCondition` |
| `tokio-util` | `0.7` | `CancellationToken` |

## Configuration surface

- `ANTHROPIC_API_KEY` — required at runtime
- Optional CLI argument: research topic string (defaults to Rust type system topic)
- `loop_with_max(name, body, exit_condition, 2)` — change `2` to allow more critic iterations

## How to modify safely

- To add a pipeline step, register another agent name in `SimpleAgentFactory` and add it to the `vec!["outline", "draft", ...]` steps list.
- To change the loop exit condition, use `ExitCondition::MaxIterations` or `ExitCondition::ToolCalled { tool_name }`.
- `Pipeline` names are for display only; `PipelineId` (auto-generated UUID) is the lookup key in the registry.

## Testing guidance

```bash
cargo check
cargo run -- "your topic here"   # requires ANTHROPIC_API_KEY
```
