# orchestrator / supervisor

Supervised multi-agent orchestration with a parent/child hierarchy.

## What it demonstrates

- Registering agents and child agents with `AgentOrchestrator`
- Attaching a `DefaultSupervisor` that automatically restarts agents on retryable errors
- Spawning agents by name via `orchestrator.spawn("name")` and interacting with `OrchestratedHandle`
- How this differs from `swarm/basic`: the orchestrator owns all agents and enforces parent/child relationships; `swarm/basic` uses peer-to-peer transfer at runtime

## Prerequisites

- Rust 1.88+
- An Anthropic API key

## Configuration (`.env`)

```
ANTHROPIC_API_KEY=sk-ant-...
```

## Run it

```bash
cargo run
```

## Testing

`cargo check` verifies the example compiles against the workspace's locked dependency versions.

## Notes

- `AgentOrchestrator` methods `add_agent` and `add_child` take `&mut self`, so the orchestrator is built imperatively rather than with a pure builder chain.
- `OrchestratedHandle::send_message` is async and returns `Result<AgentResult, AgentError>` directly — no separate `await_result` call needed for request/response usage.
- `await_result` drops the request channel, signalling clean shutdown, and is useful when you want to close the agent after the last message.
