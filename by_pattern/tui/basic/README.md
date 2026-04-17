# tui/basic

Launches a full terminal UI chat session backed by Anthropic Haiku 4.5.

The TUI is provided by `swink-agent-tui` (ratatui + crossterm). It handles rendering, keyboard input, streaming output, tool approval prompts, and optional session persistence — no UI code required in the example.

`with_default_tools()` adds `BashTool`, `ReadFileTool`, and `WriteFileTool`. Remove it for a pure chat experience.

## Run

```bash
export ANTHROPIC_API_KEY="sk-ant-..."
cargo run
```

## Key bindings

| Key | Action |
|---|---|
| `Enter` | Send message |
| `Ctrl+C` / `:q` | Quit |
| `Ctrl+Z` | Abort running generation |
| `Esc` | Cancel / close overlay |
| `↑ / ↓` | Scroll conversation |

## What it demonstrates

- `swink-agent-tui` `launch()` as the entry point for a full TUI loop
- `setup_terminal` / `restore_terminal` for raw-mode lifecycle management
- `TuiConfig::load()` for reading config from disk (theme, keybindings, session path)
- Wiring a remote model connection through `AgentOptions`

## Smoke test

The TUI requires a real controlling terminal; it cannot be verified headlessly.
Run through this 30-second check after any upstream change:

1. `cargo run` — the chat pane should render within a few seconds.
2. Type `hello, who are you?` and press `Enter`. A streamed assistant reply
   should appear.
3. Type `run bash to count .toml files in the current dir`. An approval modal
   should pop — press `y` or `Enter` to approve. The `bash` result should
   appear inline, followed by the assistant's interpretation.
4. Press `Ctrl+C` (or type `:q`) to quit. The terminal should return to its
   original state with no leftover raw-mode glitches.

If the TUI launches but immediately exits with a `Device not configured` error,
you're not running under a real tty (e.g. piped `cargo run` under CI) — the
example cannot smoke-test headlessly by design.
