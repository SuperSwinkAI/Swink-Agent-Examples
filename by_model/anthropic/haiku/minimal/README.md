# anthropic/haiku/minimal

Minimal single-turn agent using `build_remote_connection_for_model` — connects by model ID string rather than a preset key. Useful when the model ID comes from config or a CLI argument.

## What it shows

- `build_remote_connection_for_model` (string-based) vs `build_remote_connection` (preset-based)
- `ModelConnections::new` + `AgentOptions::from_connections`
- Single `prompt_text` call and iterating `result.messages`

## Run

```bash
ANTHROPIC_API_KEY=sk-ant-... cargo run
```

Or with a `.env` file containing `ANTHROPIC_API_KEY`.

## Related

- `by_model/anthropic/haiku/basic` — same pattern using the preferred `remote_preset_keys` approach
