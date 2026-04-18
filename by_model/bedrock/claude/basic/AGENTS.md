# AGENTS.md — by_model/bedrock/claude/basic

## Scope

Demonstrates two adapter construction paths for AWS Bedrock within the same binary, selected by a CLI flag. Out of scope: IAM role assumption, VPC endpoints, multi-region failover, tool use.

## Key files

- `Cargo.toml` — `swink-agent-adapters` with `bedrock` feature
- `src/main.rs` — `--direct` flag selects manual `BedrockStreamFn` construction; default uses `build_remote_connection_for_model`

## Dependencies and features

| Crate | Feature | Why |
|---|---|---|
| `swink-agent` | (default) | Core agent primitives |
| `swink-agent-adapters` | `bedrock` | `BedrockStreamFn`, Bedrock support in `build_remote_connection_for_model` |
| `tokio` | `full` | Async runtime |
| `dotenvy` | — | `.env` file loading |

## Configuration surface

| Env var | Required | Description |
|---|---|---|
| `AWS_REGION` | Yes | e.g., `us-east-1` |
| `AWS_ACCESS_KEY_ID` | Yes | IAM access key |
| `AWS_SECRET_ACCESS_KEY` | Yes | IAM secret key |
| `AWS_SESSION_TOKEN` | No | For temporary/federated credentials |

Both paths read the same env vars. In the `--direct` path they are read explicitly; in the default path the adapter reads them internally.

## How to modify safely

- **Swap model**: change `MODEL_ID` constant to any Bedrock Claude ID (e.g., `us.anthropic.claude-sonnet-4-6-20250514-v1:0`).
- **Remove the direct path**: delete the `if use_direct` branch and keep only the `build_remote_connection_for_model` call.
- **Add session token support in env path**: already handled — `build_remote_connection_for_model` reads `AWS_SESSION_TOKEN` automatically.

## Testing guidance

```bash
cargo check

# Run default (env-based) path
AWS_REGION=us-east-1 AWS_ACCESS_KEY_ID=... AWS_SECRET_ACCESS_KEY=... cargo run

# Run manual-construction path
AWS_REGION=us-east-1 AWS_ACCESS_KEY_ID=... AWS_SECRET_ACCESS_KEY=... cargo run -- --direct

# Both should print the same response. A credential error surfaces as an AWS SDK error before any agent activity.
```
