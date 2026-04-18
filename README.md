# Swink-Agent-Examples

Standalone examples for the [Swink-Agent](https://github.com/SuperSwinkAI/Swink-Agent) framework — a pure-Rust library for LLM-powered agentic loops.

## Structure

```
by_model/                    provider & model-specific examples
  anthropic/haiku/           Claude Haiku (basic + minimal)
  openai/gpt-5/              OpenAI GPT-5
  mistral/mistral-large/     Mistral Large
  ollama/llama3/             Ollama (local, no API key)
  local/{gemma4-e2b,         On-device inference via llama.cpp (GGUF)
         gemma4-e4b,
         smollm3}/
  google/gemini/             Google Gemini (GEMINI_API_KEY)
  bedrock/claude/            AWS Bedrock — env-based + direct SigV4 construction
  openai_compat/multi/       Azure OpenAI, xAI Grok, LM Studio — one binary, --backend flag

by_pattern/                  agentic design patterns
  tool_use/                  builtin tools + FnTool + approval callback
  mock_provider/             testing with a mock StreamFn
  custom_adapter/            custom StreamFn implementation
  swarm/                     peer-to-peer agent transfer (TransferToAgentTool)
  tui/{basic,yolo}           terminal UI via swink-agent-tui
  mcp/context7_docs/         MCP client (HTTP/SSE, bearer auth) + ToolDenyListPolicy + AuditLogger
  production_guardrails/     Budget + MaxTurns + PromptGuard + PII + ContentFilter +
                             LoopDetection + ToolMiddleware + StreamMiddleware + MetricsCollector
  persistent_chat/           JsonlSessionStore + checkpointing + sliding-window compaction +
                             steering + SessionState + plan mode
  context_versioning/        VersioningTransformer + ContextSummarizer + InMemoryVersionStore
  hot_reload/                ScriptTool + ToolWatcher (live-reload from directory) + SandboxPolicy
  orchestrator/              AgentOrchestrator + DefaultSupervisor + SupervisorPolicy
  pipeline/                  Pipeline (sequential → parallel → loop) + FileArtifactStore
  parallel_tools/            ToolExecutionPolicy (concurrent / sequential / priority)
  prompt_caching/            CacheConfig + CacheStrategy + cache token reporting
  observability/             OTel tracing + MetricsCollector + tiktoken TokenCounter
  model_fallback/            ModelFallback with loud logging — honest framing of tradeoffs

usecases/                    goal-oriented examples
  summarization/             single-agent document summarization
  research/                  web research + file output (WebPlugin)
  eval_harness/              EvalRunner + EvalSet + trajectory matching + AuditLogger
```

Each example is its own Cargo project — `cd` into the folder and `cargo run`.

Each example includes:
- **`README.md`** — what it demonstrates, env vars, how to run, expected output
- **`AGENTS.md`** — guidance for AI coding agents: scope, key files, deps, config surface, how to modify safely

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
   - `by_model/` paths must be three levels deep: `<provider>/<model>/<name>/`.
   - `by_pattern/` and `usecases/` paths must be two levels deep: `<pattern>/<name>/`.
2. `cargo init` inside it.
3. Add `swink-agent = "0.7"` and any needed crates to `Cargo.toml`. Set `edition = "2024"`, `rust-version = "1.88"`, `publish = false`.
4. Write `README.md` covering: what it demonstrates, prerequisites, configuration (`.env` lines), how to run, testing, and notes.
5. Write `AGENTS.md` covering: scope, key files, dependencies and features, configuration surface, how to modify safely, testing guidance.
6. Verify it compiles: `cargo check` from the example directory, then `cargo check --workspace` from the repo root.

## License

[MIT](LICENSE)
