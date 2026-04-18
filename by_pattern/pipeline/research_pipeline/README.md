# pipeline / research_pipeline

3-stage research pipeline: outline → draft → critic review loop.

## What it demonstrates

- Building sequential pipelines with `Pipeline::sequential_with_context` (output is forwarded as input to each next step)
- Building loop pipelines with `Pipeline::loop_with_max` and `ExitCondition::output_contains`
- Registering agent builder functions with `SimpleAgentFactory`
- Registering pipelines in `PipelineRegistry` and executing them with `PipelineExecutor`
- Reading `PipelineOutput` fields (`final_response`, `steps`, per-step `usage`)

## Prerequisites

- Rust 1.88+
- An Anthropic API key

## Configuration (`.env`)

```
ANTHROPIC_API_KEY=sk-ant-...
```

## Run it

```bash
# Default topic
cargo run

# Custom topic
cargo run -- "zero-cost abstractions in systems languages"
```

## Testing

```bash
cargo check
```

## Notes

- `Pipeline::sequential` vs `sequential_with_context`: the latter accumulates conversation history across steps so each agent sees prior output in context; the former passes only the previous step's text output as the next step's user message.
- The critic loop exits when the agent's response matches the regex `APPROVED` or after 2 iterations (`loop_with_max`).
- `PipelineExecutor::run` takes a `CancellationToken` — pass a cloned token to allow cooperative cancellation from a signal handler.
