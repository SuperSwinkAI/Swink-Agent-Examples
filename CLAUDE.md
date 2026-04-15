# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Purpose

This is the examples repository for [Swink-Agent](https://github.com/SuperSwinkAI/Swink-Agent), a pure-Rust library for LLM-powered agentic loops. Each example is a standalone Rust binary (its own `Cargo.toml`) demonstrating a specific pattern, provider, or use case.

## Repository layout

```
by_model/<provider>/<model>/<example>/      # provider/model-specific examples
by_pattern/<pattern>/<example>/             # agentic patterns (swarm, chain-of-thought, routing, etc.)
usecases/<objective>/<example>/             # goal-oriented examples (summarization, code-gen, RAG, etc.)
```

Every example is an independent Cargo project with its own `Cargo.toml` that depends on `swink-agent` and optionally `swink-agent-adapters`, `swink-agent-memory`, `swink-agent-policies`, etc.

## Swink-Agent quick reference

The upstream library lives at `../Swink-Agent` (path dependency) or on crates.io as `swink-agent`.

**Key crates and what they provide:**

| Crate | Purpose |
|---|---|
| `swink-agent` | Core: `Agent`, `AgentOptions`, `StreamFn`, tool system, policies, context management |
| `swink-agent-adapters` | Provider adapters (feature-gated): `anthropic`, `openai`, `ollama`, `gemini`, `azure`, `bedrock`, `mistral`, `xai`, `proxy` |
| `swink-agent-memory` | JSONL session persistence (`SessionStore`, `JsonlSessionStore`) |
| `swink-agent-policies` | Policy implementations: budget, max-turns, deny-list, sandbox, loop-detection, etc. |
| `swink-agent-local-llm` | Local inference via `mistralrs` with hardware-backend features (`metal`, `cuda`, etc.) |
| `swink-agent-tui` | Terminal UI (ratatui + crossterm) |

**Adapters are opt-in via feature flags.** A typical `Cargo.toml` for an Anthropic example:

```toml
[dependencies]
swink-agent = "0.7.1"
swink-agent-adapters = { version = "0.7.1", features = ["anthropic"] }
tokio = { version = "1", features = ["full"] }
```

**Core API patterns:**
- `Agent::new(options)` → configure with `AgentOptions`
- `StreamFn` trait — all LLM interaction is async streaming
- `AgentTool` / `FnTool` / `IntoTool` — tool registration
- `builtin_tools()` — returns `[BashTool, ReadFileTool, WriteFileTool]` (requires `builtin-tools` feature, on by default)
- Policies via `PreTurnPolicy`, `PostTurnPolicy`, `PreDispatchPolicy`, `PostLoopPolicy` traits

## Commands

```bash
# Build a specific example
cd by_model/anthropic/haiku/basic && cargo build

# Run a specific example
cd by_model/anthropic/haiku/basic && cargo run

# Check all examples compile (from repo root)
for dir in $(find . -name "Cargo.toml" -not -path "./.git/*" -exec dirname {} \;); do
  (cd "$dir" && cargo check) || echo "FAILED: $dir"
done

# Run tests in a specific example
cd usecases/summarization/basic && cargo test

# Format / lint a specific example
cd <example-dir> && cargo fmt -- --check
cd <example-dir> && cargo clippy -- -D warnings
```

## Conventions

- **MSRV**: Rust 1.88, edition 2024
- Each example must be self-contained — no shared lib crate across examples
- Each example should have a `README.md` explaining what it demonstrates and any required env vars (e.g., `ANTHROPIC_API_KEY`)
- Prefer `build_remote_connection_for_model(model_id)` over constructing `RemotePresetKey` manually (e.g., `build_remote_connection_for_model("claude-haiku-4-5-20251001")`)
- Use `OllamaStreamFn::new(base_url)` + `ModelConnection::new(ModelSpec, stream_fn)` directly for Ollama (it is a `local` provider, not in the remote catalog)
- Use `tokio` as the async runtime (all Swink-Agent crates assume tokio)
- Examples targeting real providers need API keys via environment variables — never hardcode credentials
- When adding a new example, ensure it compiles with `cargo check` before committing

## Provider adapter features

When writing an example for a specific provider, enable only that adapter's feature:

| Provider | Feature flag | StreamFn type |
|---|---|---|
| Anthropic | `anthropic` | `AnthropicStreamFn` |
| OpenAI | `openai` | `OpenAiStreamFn` |
| Ollama | `ollama` | `OllamaStreamFn` |
| Google Gemini | `gemini` | `GeminiStreamFn` |
| Azure OpenAI | `azure` | `AzureStreamFn` |
| AWS Bedrock | `bedrock` | `BedrockStreamFn` |
| Mistral | `mistral` | `MistralStreamFn` |
| xAI | `xai` | `XAiStreamFn` |
| Generic proxy | `proxy` | `ProxyStreamFn` |
