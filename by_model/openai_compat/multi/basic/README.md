# by_model/openai_compat/multi/basic

Single binary that can talk to three different OpenAI-compatible backends — LM Studio (local), Azure OpenAI, or xAI/Grok — by passing the backend name as a CLI argument. Demonstrates how the same agent loop works across providers that share the OpenAI wire protocol.

## What it demonstrates

- `OpenAiStreamFn::new(base_url, auth)` — generic OpenAI-compatible HTTP client (used for LM Studio)
- `AzureStreamFn::new(base_url, AzureAuth::ApiKey(key))` — Azure OpenAI endpoint
- `XAiStreamFn::new(base_url, api_key)` — xAI (Grok) API
- `ModelSpec::new(provider, model_id)` + `ModelConnection::new(spec, Arc::new(stream_fn))`
- Runtime backend selection via `std::env::args().nth(1)`
- `swink-agent-adapters` with multiple features: `openai`, `azure`, `xai`

## Prerequisites

- One of:
  - LM Studio running locally (default, no API key needed)
  - Azure OpenAI deployment with endpoint and key
  - xAI API key for Grok
- Rust 1.88+

## Configuration

```env
# For LM Studio (both optional — defaults shown)
LM_STUDIO_BASE_URL=http://localhost:1234/v1
LM_STUDIO_MODEL=local-model

# For Azure
AZURE_BASE_URL=https://your-resource.openai.azure.com/openai/deployments/gpt-4o
AZURE_API_KEY=your-azure-key

# For xAI
XAI_API_KEY=your-xai-key
```

## Run it

```bash
# LM Studio (default)
cargo run

# Azure OpenAI
cargo run -- azure

# xAI / Grok
cargo run -- xai
```

## Testing

Each run prints a header line showing the active backend, then the prompt, then one paragraph from the model explaining "OpenAI-compatible". LM Studio will work offline with no API keys set. For Azure/xAI, verify the response appears without HTTP errors.

## Notes

- LM Studio ignores the auth string (`"lm-studio"`) — it accepts any value.
- Any unrecognised backend name falls through to the LM Studio branch as a safe default.
- Azure requires the full deployment URL including the deployment name segment, not just the base resource URL.
- The xAI model ID `"grok-4.20-0309-non-reasoning"` — update if xAI changes the model slug.
