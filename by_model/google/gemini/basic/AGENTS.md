# AGENTS.md — by_model/google/gemini/basic

## Scope

Shows the minimal path to run a Gemini model via Swink-Agent using the env-based convenience function. Out of scope: tools, multi-model routing, streaming callbacks, retries.

## Key files

- `Cargo.toml` — declares `swink-agent` + `swink-agent-adapters` with `gemini` feature
- `src/main.rs` — full example: connect, configure agent, send two prompts, print results

## Dependencies and features

| Crate | Feature | Why |
|---|---|---|
| `swink-agent` | (default) | `Agent`, `AgentOptions`, `ModelConnections`, message types |
| `swink-agent-adapters` | `gemini` | `GeminiStreamFn` and `build_remote_connection_for_model` support for Gemini |
| `tokio` | `full` | Async runtime required by Swink-Agent |
| `dotenvy` | — | Loads `.env` file for `GEMINI_API_KEY` |

## Configuration surface

| Env var | Required | Description |
|---|---|---|
| `GEMINI_API_KEY` | Yes | Google AI Studio API key; read automatically by `build_remote_connection_for_model` |

Credentials are never hardcoded. `dotenvy::dotenv().ok()` silently skips if no `.env` exists, so plain env exports also work.

## How to modify safely

- **Swap model**: change `"gemini-3-flash-preview"` to any other Gemini model ID in `main.rs`. No other code changes needed.
- **Add a tool**: import `FnTool`/`IntoTool` from `swink_agent`, construct it, and pass it as the second `ModelConnection` in `ModelConnections::new(connection, vec![tool])`.
- **Change system prompt**: edit the string passed to `AgentOptions::from_connections`.
- **Add prompts**: extend the array in the `for prompt in [...]` loop.

## Testing guidance

```bash
# Verify the code compiles (no API call made)
cargo check

# Run against the real Gemini API
GEMINI_API_KEY=your-key cargo run

# Expected: two prompt lines prefixed with ">>> " followed by Gemini's responses.
# A missing or invalid API key prints "Error connecting to Gemini: ..." and exits non-zero.
```
