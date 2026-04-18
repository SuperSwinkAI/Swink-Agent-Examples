# parallel_tools / fan_out_fetch

Fan-out tool calls with three `ToolExecutionPolicy` modes.

## What it demonstrates

- Defining multiple `FnTool` tools that simulate slow IO (500 ms sleep each)
- Setting `ToolExecutionPolicy::Sequential`, `::Concurrent`, and `::Priority` via `AgentOptions::with_tool_execution_policy`
- Measuring wall-clock elapsed time to observe the speedup from concurrent dispatch
- Using `Priority` mode to schedule one tool (weather) before the others

## Prerequisites

- Rust 1.88+
- An Anthropic API key

## Configuration (`.env`)

```
ANTHROPIC_API_KEY=sk-ant-...
```

## Run it

```bash
# Default: concurrent (fastest)
cargo run

# Sequential (~2 s — all four tools run one-at-a-time)
cargo run -- --mode sequential

# Concurrent (~0.5 s — all tools run in parallel, default)
cargo run -- --mode concurrent

# Priority — weather runs in batch 1, rest in batch 2
cargo run -- --mode priority
```

## Testing

```bash
cargo check
```

## Notes

- `ToolExecutionPolicy::Priority(Arc<PriorityFn>)` groups tools by score (higher = earlier). Tools within the same score group still run concurrently.
- The agent is given a system prompt that instructs it to use ALL tools, so the model should emit all four tool calls in a single turn.
- Elapsed time is measured from just before `agent.prompt_text` to just after — it includes the model round-trip in addition to tool execution time.
