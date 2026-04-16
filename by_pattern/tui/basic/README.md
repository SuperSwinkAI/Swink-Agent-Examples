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
