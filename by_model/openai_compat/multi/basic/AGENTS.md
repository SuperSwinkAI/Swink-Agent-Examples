# AGENTS.md — by_model/openai_compat/multi/basic

## Scope

Shows how to wire three different OpenAI-compatible backends into Swink-Agent using a single match on a CLI argument. Out of scope: automatic retries, load balancing across providers, streaming output, tool use.

## Key files

- `Cargo.toml` — `swink-agent-adapters` with `openai`, `azure`, and `xai` features
- `src/main.rs` — `backend` variable drives which `StreamFn` is constructed; all three branches share the same `AgentOptions`/`Agent` code

## Dependencies and features

| Crate | Feature | Why |
|---|---|---|
| `swink-agent` | (default) | Core agent primitives |
| `swink-agent-adapters` | `openai` | `OpenAiStreamFn` for LM Studio (and generic OpenAI-compat servers) |
| `swink-agent-adapters` | `azure` | `AzureStreamFn`, `AzureAuth` |
| `swink-agent-adapters` | `xai` | `XAiStreamFn` |
| `tokio` | `full` | Async runtime |
| `dotenvy` | — | `.env` file loading |

## Configuration surface

| Env var | Required for | Default |
|---|---|---|
| `LM_STUDIO_BASE_URL` | lmstudio | `http://localhost:1234/v1` |
| `LM_STUDIO_MODEL` | lmstudio | `local-model` |
| `AZURE_BASE_URL` | azure | — (required) |
| `AZURE_API_KEY` | azure | — (required) |
| `XAI_API_KEY` | xai | — (required) |

Credentials for unused backends are never read, so it is safe to run the lmstudio path with no keys set.

## How to modify safely

- **Add another backend**: add a new match arm, construct its `StreamFn` and `ModelSpec`, assign a `ModelConnection`, then the rest of the code is unchanged.
- **Change the Azure model**: update `"gpt-4o"` in the `ModelSpec::new("azure", ...)` call.
- **Use API key auth for LM Studio**: change `OpenAiStreamFn::new(base_url, "lm-studio")` to `OpenAiStreamFn::new(base_url, api_key_string)`.
- **Change the prompt**: edit the `let prompt = "..."` line — it is shared across all backends.

## Testing guidance

```bash
cargo check

# Run with LM Studio (must be running locally)
cargo run

# Run with Azure (requires env vars)
AZURE_BASE_URL=https://... AZURE_API_KEY=... cargo run -- azure

# Run with xAI
XAI_API_KEY=... cargo run -- xai

# Verify: each prints "Backend: <name>", the prompt, and a one-paragraph response.
```
