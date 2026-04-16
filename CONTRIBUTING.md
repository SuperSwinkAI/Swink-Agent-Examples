# Contributing to Swink-Agent-Examples

This guide exists to save both sides time.

## The Standard

**You must understand your code.** If you cannot explain what your example demonstrates and how it uses the Swink-Agent API, your PR will be closed.

Using AI to write code is fine. Submitting AI-generated code without understanding it is not.

If you use an agent, run it from the repo root so it picks up `AGENTS.md` automatically. Your agent must follow the rules in that file.

## Contribution Gate

PRs from new contributors are auto-closed by default. Issues are open to everyone.

Maintainers review auto-closed PRs and reopen worthwhile ones. Reply `lgtm` on any issue or PR from a contributor to grant them PR rights going forward.

## Quality Bar for Issues

Use one of the [GitHub issue templates](https://github.com/SuperSwinkAI/Swink-Agent-Examples/issues/new/choose).

Keep it short and concrete:

- One screen or less.
- State the bug or example request clearly.
- Explain what pattern or use case it demonstrates that isn't already covered.
- If you want to implement it yourself, say so.

## Blocking

If you spam the tracker with agent-generated issues or PRs, your GitHub account will be permanently blocked.

## Prerequisites

- **Rust 1.88+** (MSRV). Install via [rustup](https://rustup.rs).
- Add required toolchain components:
  ```bash
  rustup component add clippy rustfmt
  ```
- Clone this repo:
  ```bash
  git clone https://github.com/SuperSwinkAI/Swink-Agent-Examples.git
  ```
- Copy `.env.example` to `.env` and populate the API keys for the providers your example targets.

## Adding a New Example

1. **Open an issue first.** Describe what the example demonstrates and where it would live (`by_model/`, `by_pattern/`, or `usecases/`). Get feedback before writing code.
2. Create a new directory under the appropriate category.
3. `cargo init` inside it and add `swink-agent` and any needed adapter crates to `Cargo.toml`.
4. Include a `README.md` describing what it demonstrates and any required env vars.
5. Verify it compiles: `cargo check`.

## Before Submitting a PR

Three requirements:

1. **Open an issue first.** PRs without a linked issue are auto-closed.
2. **Get `lgtm` approval** (see Contribution Gate above).
3. Run these locally before opening:

```bash
cargo fmt --check                        # formatting
cargo clippy --workspace -- -D warnings  # zero warnings
cargo check --workspace                  # all examples compile
```

All three must pass.

## Pull Request Process

1. Open a PR against `main` with a clear title and description.
2. Describe *what* the example demonstrates and *why* it belongs in the repo.
3. Reference the related issue with `Closes #<issue>` or `Related to #<issue>`.
4. All CI checks must pass before merge.
5. At least one maintainer review is required.

## Commit Messages

Concise, imperative-mood subject lines:

```
Add by_pattern/chain_of_thought/basic example
Fix by_model/openai/gpt-4o/basic compile error on Windows
Update tui/yolo to swink-agent 0.7.4
```

No ticket numbers in commit messages — link issues in the PR description.

## Branch Naming

| Type | Pattern | Example |
|---|---|---|
| New example | `example/<path>` | `example/by_pattern/rag-basic` |
| Bug fix | `fix/<short-description>` | `fix/tui-basic-windows-build` |
| Docs | `docs/<short-description>` | `docs/improve-swarm-readme` |
| Chore | `chore/<short-description>` | `chore/bump-swink-agent-0.7.4` |

Branch off `main`. One example or concern per PR.

## Example Conventions

See `AGENTS.md` for the full list. Key rules:

- Each example is self-contained — its own `Cargo.toml`, no shared lib crate.
- Each example must have a `README.md` explaining what it demonstrates and all required env vars.
- Use `build_remote_connection_for_model(model_id)` for remote providers.
- Never hardcode API keys or credentials — use environment variables.
- Prefer `dotenvy::dotenv().ok()` for loading `.env` in examples.
- `publish = false` in every example's `Cargo.toml`.
