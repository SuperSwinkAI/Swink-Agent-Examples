# AGENTS.md

This file provides guidance to AI coding agents when working with code in this repository.

## Purpose

This is the [Swink-Agent-Examples](https://github.com/SuperSwinkAI/Swink-Agent-Examples) repository — standalone examples for [Swink-Agent](https://github.com/SuperSwinkAI/Swink-Agent), a pure-Rust library for LLM-powered agentic loops. Each example is a standalone Rust binary (its own `Cargo.toml`) demonstrating a specific pattern, provider, or use case.

## Repository layout

```
by_model/<provider>/<model>/<example>/      # provider/model-specific examples
by_pattern/<pattern>/<example>/             # agentic patterns (swarm, chain-of-thought, routing, etc.)
usecases/<objective>/<example>/             # goal-oriented examples (summarization, code-gen, RAG, etc.)
```

Every example is an independent Cargo project with its own `Cargo.toml` that depends on `swink-agent` and optionally `swink-agent-adapters`, `swink-agent-memory`, `swink-agent-policies`, etc.

Each example ships three documentation files:
- **`README.md`** — human-facing: what it shows, env vars, run instructions, expected output.
- **`AGENTS.md`** — agent-facing: scope, key files, deps/features, configuration surface, how to modify safely, testing guidance.
- **`src/main.rs`** — the implementation.

## Swink-Agent quick reference

The upstream library is on crates.io as `swink-agent`. Source at [SuperSwinkAI/Swink-Agent](https://github.com/SuperSwinkAI/Swink-Agent).

**Key crates and what they provide:**

| Crate | Purpose |
|---|---|
| `swink-agent` | Core: `Agent`, `AgentOptions`, `StreamFn`, tool system, policies, context management |
| `swink-agent-adapters` | Provider adapters (feature-gated): `anthropic`, `openai`, `ollama`, `gemini`, `azure`, `bedrock`, `mistral`, `xai`, `proxy` |
| `swink-agent-memory` | JSONL session persistence (`SessionStore`, `JsonlSessionStore`) and context compaction |
| `swink-agent-policies` | Policy implementations: `budget`, `max-turns`, `deny-list`, `sandbox`, `loop-detection`, `prompt-guard`, `pii`, `content-filter`, `audit` |
| `swink-agent-mcp` | MCP client: server connection, tool discovery, name prefixing (`McpManager`, `McpTransport`) |
| `swink-agent-patterns` | Pipeline orchestration: `Pipeline`, `PipelineExecutor`, `SimpleAgentFactory` (feature: `pipelines`) |
| `swink-agent-artifacts` | Versioned artifact storage: `FileArtifactStore`, `InMemoryArtifactStore` |
| `swink-agent-eval` | Evaluation framework: `EvalRunner`, `EvalSet`, trajectory matching, budget/response scoring |
| `swink-agent-local-llm` | On-device inference via llama.cpp (GGUF); hardware features: `metal`, `cuda`, `vulkan` |
| `swink-agent-tui` | Terminal UI (ratatui + crossterm) |

**Adapters are opt-in via feature flags.** A typical `Cargo.toml` for an Anthropic example:

```toml
[package]
name = "my-example"
version = "0.1.0"
edition = "2024"
rust-version = "1.88"
publish = false

[dependencies]
swink-agent = "0.7"
swink-agent-adapters = { version = "0.7", features = ["anthropic"] }
tokio = { version = "1", features = ["full"] }
dotenvy = "0.15"
```

Pin deps to `"0.7"` (not `"0"`). Always include `dotenvy = "0.15"` and call `dotenvy::dotenv().ok();` as the first line of `main`.

**Core API patterns:**
- `Agent::new(options)` → configure with `AgentOptions`
- `build_remote_connection_for_model("model-id")` → builds a `ModelConnection` reading the provider's env var automatically
- `ModelConnections::new(connection, vec![])` + `AgentOptions::from_connections(system_prompt, connections)`
- `StreamFn` trait — all LLM interaction is async streaming
- `AgentTool` / `FnTool` / `IntoTool` — tool registration
- `builtin_tools()` — returns `[BashTool, ReadFileTool, WriteFileTool, EditFileTool]` (requires `builtin-tools` feature, on by default)
- Policies register individually via `AgentOptions::with_pre_turn_policy(p)`, `with_post_turn_policy(p)`, `with_pre_dispatch_policy(p)`, `with_post_loop_policy(p)` — there is **no** `with_policies(vec![...])` method
- `ToolMiddleware` wraps an `AgentTool` and is itself an `AgentTool` — pass via `with_tools()`; **not in prelude**, import explicitly
- `StreamMiddleware` wraps a `StreamFn` and is itself a `StreamFn` — use in `ModelConnection::new`
- `ApprovalMode` — **not in prelude**, import explicitly: `use swink_agent::ApprovalMode;`
- Error type: always `Box<dyn std::error::Error>`, never `anyhow`

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
- Each example must have both a `README.md` and an `AGENTS.md` (see per-example file sections above)
- `by_model/` paths are three levels deep: `<provider>/<model>/<name>/`; `by_pattern/` and `usecases/` are two levels
- Pin `swink-agent` deps to `"0.7"`, not `"0"`
- Prefer `build_remote_connection_for_model(model_id)` for remote providers — it reads the provider's credential env var automatically
- Use `OllamaStreamFn::new(base_url)` + `ModelConnection::new(ModelSpec::new("local", model), stream_fn)` for Ollama (local provider, not in remote catalog)
- Use `tokio` as the async runtime; never spawn threads around agent calls
- Never hardcode credentials — always read from env vars or `.env` via `dotenvy`
- When adding a new example, run `cargo check` in the example dir, then `cargo check --workspace` from the repo root
- `#[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>>` — no `anyhow`
- Plain `println!` for output; add `tracing_subscriber` only when the example specifically demonstrates observability

## Known API gotchas (discovered authoring these examples)

- **No `with_policies` method** — register each policy individually via `with_pre_turn_policy` / `with_post_turn_policy` / `with_pre_dispatch_policy` / `with_post_loop_policy`
- **`ToolMiddleware` and `ApprovalMode` are not in `swink_agent::prelude`** — import explicitly (issues [#652](https://github.com/SuperSwinkAI/Swink-Agent/issues/652), [#655](https://github.com/SuperSwinkAI/Swink-Agent/issues/655))
- **`tiktoken` feature ships no wrapper** — implement `TokenCounter` locally if you need tiktoken (issue [#651](https://github.com/SuperSwinkAI/Swink-Agent/issues/651))
- **`FnTool` has no async execute variant** — use `with_execute_simple` for sync closures; implement `AgentTool` manually for async (issue [#653](https://github.com/SuperSwinkAI/Swink-Agent/issues/653))
- **`McpTransport::Sse` is bearer-token only** — no arbitrary header auth (issue [#654](https://github.com/SuperSwinkAI/Swink-Agent/issues/654))
- **`ScriptTool` TOML schema uses `[parameters_schema]`** — not `[parameters.properties]` (issue [#656](https://github.com/SuperSwinkAI/Swink-Agent/issues/656))

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
