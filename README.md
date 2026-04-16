# Swink-Agent-Examples

Standalone examples for the [Swink-Agent](https://github.com/SuperSwinkAI/Swink-Agent) framework — a pure-Rust library for LLM-powered agentic loops.

## Structure

```
by_model/          provider & model-specific examples
  anthropic/
  openai/
  ollama/
  mistral/
  ...

by_pattern/        agentic design patterns
  swarm/           multi-agent swarm coordination
  ...

usecases/          goal-oriented examples
  summarization/
  code-gen/
  ...
```

Each example is its own Cargo project — `cd` into the folder and `cargo run`.

## Quick start

```bash
# Clone the repo
git clone https://github.com/SuperSwinkAI/Swink-Agent-Examples.git
cd Swink-Agent-Examples

# Pick an example
cd by_model/anthropic/haiku/basic

# Set your API key
export ANTHROPIC_API_KEY="sk-ant-..."

# Run it
cargo run
```

## Prerequisites

- **Rust 1.88+** (edition 2024)
- API keys for whichever provider the example targets (see each example's README)

## Adding an example

1. Create a new directory under the appropriate category (`by_model/`, `by_pattern/`, or `usecases/`).
2. `cargo init` inside it.
3. Add `swink-agent` and any needed crates to `Cargo.toml`.
4. Include a `README.md` describing what it demonstrates and required env vars.
5. Verify it compiles: `cargo check`.

## License

[MIT](LICENSE)
