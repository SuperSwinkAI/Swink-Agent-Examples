# by_model/bedrock/claude/basic

Two-path example for running Claude on AWS Bedrock via Swink-Agent. The default path uses the env-based convenience function; the `--direct` flag demonstrates manual `BedrockStreamFn` construction for cases where you need explicit credential control.

## What it demonstrates

- `build_remote_connection_for_model(model_id)?` — automatic credential pickup from env vars
- `BedrockStreamFn::new(region, access_key, secret_key, session_token)` — manual construction
- `ModelSpec::new("bedrock", model_id)` + `ModelConnection::new(spec, Arc::new(stream_fn))`
- CLI flag switching between two adapter construction paths
- `swink-agent-adapters` with `features = ["bedrock"]`

## Prerequisites

- AWS account with Bedrock model access enabled for `us.anthropic.claude-haiku-4-5-20251001-v1:0`
- `AWS_REGION`, `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY` env vars (plus optional `AWS_SESSION_TOKEN`)
- Rust 1.88+

## Configuration

```env
AWS_REGION=us-east-1
AWS_ACCESS_KEY_ID=AKIA...
AWS_SECRET_ACCESS_KEY=your-secret
# AWS_SESSION_TOKEN=optional-if-using-temporary-credentials
```

## Run it

```bash
# Default: env-based connection
cargo run

# Manual BedrockStreamFn construction
cargo run -- --direct
```

## Testing

Both paths should produce the same response to the Bedrock question. If credentials are missing or access is not granted you will see an error from the AWS SDK before any output. Verify the model is enabled in the AWS console under Bedrock > Model access.

## Notes

- `BedrockStreamFn` requires us-prefixed cross-region inference IDs (e.g., `us.anthropic...`).
- Costs depend on token volume; Haiku is the cheapest Claude tier on Bedrock.
- Temporary credentials (`AWS_SESSION_TOKEN`) are fully supported in both paths.
