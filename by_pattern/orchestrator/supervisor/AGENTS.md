# AGENTS.md — orchestrator/supervisor

## Scope

Demonstrates hierarchical multi-agent orchestration with automatic fault recovery using `AgentOrchestrator` and `DefaultSupervisor`.

## Key files

- `src/main.rs` — sets up a two-agent hierarchy (planner → executor), attaches a supervisor, and drives both agents
- `Cargo.toml` — depends on `swink-agent` and `swink-agent-adapters`

## Dependencies and features

| Crate | Version | Notes |
|---|---|---|
| `swink-agent` | `0.7` | `AgentOrchestrator`, `DefaultSupervisor`, `AgentOptions`, `ModelConnections` |
| `swink-agent-adapters` | `0.7` (feature `anthropic`) | `build_remote_connection_for_model` |

## Configuration surface

- `ANTHROPIC_API_KEY` — required at runtime
- `DefaultSupervisor::new(max_restarts)` — change the restart budget
- `AgentOrchestrator::with_channel_buffer(n)` — tune the request channel buffer (default 32)

## How to modify safely

- To add more child agents, call `orchestrator.add_child("name", "parent", factory)` before any `spawn` calls.
- To use a custom supervisor policy, implement `SupervisorPolicy` and pass it to `with_supervisor`.
- Factories are `Fn() -> AgentOptions + Send + Sync + 'static` — clone connection state inside the closure, not outside.

## Testing guidance

```bash
cargo check   # compile-time validation
cargo run     # requires ANTHROPIC_API_KEY
```
