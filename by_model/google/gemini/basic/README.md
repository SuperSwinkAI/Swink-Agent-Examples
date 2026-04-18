# by_model/google/gemini/basic

Minimal two-turn example that sends two prompts to Google Gemini Flash using the Swink-Agent Gemini adapter. The agent reads `GEMINI_API_KEY` from the environment and returns plain text, with clean error reporting if the connection cannot be established.

## What it demonstrates

- `build_remote_connection_for_model("gemini-3-flash-preview")` — env-based Gemini setup
- `swink-agent-adapters` with `features = ["gemini"]`
- `ModelConnections::new` / `AgentOptions::from_connections` / `Agent::new`
- `agent.prompt_text(prompt).await?` for turn-by-turn prompting
- `ContentBlock::extract_text` for extracting assistant output from messages
- Explicit `eprintln!` + early return on connection error (no provider fallback)

## Prerequisites

- `GEMINI_API_KEY` — Google AI Studio API key
- Rust 1.88+

## Configuration

Copy to a `.env` file in this directory (or export before running):

```env
GEMINI_API_KEY=your-google-ai-studio-key
```

## Run it

```bash
cargo run
```

## Testing

Expected output: two prompt/response pairs printed to stdout. Each response answers a short Rust question in plain prose. If `GEMINI_API_KEY` is missing or invalid you will see an `Error connecting to Gemini:` line and a non-zero exit code.

## Notes

- Model ID `"gemini-3-flash-preview"` maps to the current Gemini Flash preview; swap for `"gemini-3.1-pro-preview"` to use the Pro tier (higher cost, slower).
- Each call is a fresh turn in the same agent loop, so context from prompt 1 is available to prompt 2.
