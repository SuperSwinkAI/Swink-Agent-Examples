# tui/yolo

TUI agent with all built-in tools and zero approval prompts.

"You Only Live Once" — every tool call (bash, file read, file write, file edit) is auto-approved. The agent never pauses to ask permission. Backed by Claude Sonnet 4.6.

Use this when you trust the model and want maximum autonomy. Avoid it on sensitive machines or with untrusted prompts.

## Run

```bash
export ANTHROPIC_API_KEY="sk-ant-..."
cargo run
```

## What it demonstrates

- `ApprovalMode::Bypassed` on `AgentOptions` — suppresses all tool-approval overlays
- `launch()` propagates the approval mode to both the agent dispatch loop and the TUI automatically (no manual sync needed)
- `with_default_tools()` wiring all built-in tools in a single call

## Key bindings

| Key | Action |
|---|---|
| `Enter` | Send message |
| `Ctrl+C` / `:q` | Quit |
| `Ctrl+Z` | Abort running generation |
| `Esc` | Cancel / close overlay |
| `↑ / ↓` | Scroll conversation |
