# Swarm — Basic

Multi-agent swarm with a triage agent that routes requests to specialist agents (researcher, writer) using Swink-Agent's transfer/handoff system.

## Requirements

- `ANTHROPIC_API_KEY`

## Run

```bash
cargo run
```

## How it works

1. A **triage** agent receives the user prompt.
2. Based on the request, it transfers to either a **researcher** or **writer** agent.
3. The specialist handles the request and returns the result.

This demonstrates `AgentRegistry` and the `transfer` feature for multi-agent coordination.
