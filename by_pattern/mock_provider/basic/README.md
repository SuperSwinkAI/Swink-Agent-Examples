# mock_provider/basic

Demonstrates the core `Agent` API using a hand-rolled `MockStreamFn` — no real LLM provider or API key required.

## What it shows

- Implementing the `StreamFn` trait with scripted responses
- Building `AgentOptions` with `new_simple`
- Sending a prompt with `agent.prompt_text()` and reading `result.assistant_text()`

## Run

```bash
cargo run
```

## Next steps

Replace `MockStreamFn` with a real adapter from `swink-agent-adapters`:

```toml
swink-agent-adapters = { version = "0.5", features = ["anthropic"] }
```

```rust
use swink_agent_adapters::{build_remote_connection, remote_preset_keys};
let connection = build_remote_connection(remote_preset_keys::anthropic::HAIKU_45)?;
```

See `by_model/anthropic/haiku/basic` for a complete real-provider example.
