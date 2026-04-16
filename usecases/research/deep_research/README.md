# deep-research

A multi-agent deep-research pipeline with a TUI frontend, demonstrating
`SaveArtifactTool`, `LoadArtifactTool`, and `ListArtifactsTool` for
inter-agent artifact sharing.

## What it demonstrates

- **`artifact-tools` feature** — `SaveArtifactTool`, `LoadArtifactTool`,
  `ListArtifactsTool`, and the `artifact_tools()` convenience constructor
- **Custom `ArtifactStore`** — `InMemoryArtifactStore` implementing the
  `ArtifactStore` trait, shared across agents via `Arc`
- **Multi-agent pipeline** — three specialised sub-agents run as tool calls
  on a coordinator; artifacts are the hand-off mechanism between them
- **TUI integration** — `swink-agent-tui` with `ApprovalMode::Bypassed`;
  each pipeline phase surfaces as a distinct tool call the user can watch
- **`with_state_entry("session_id", ...)`** — wires the shared session ID
  into each agent's state so artifact tools read/write the same bucket

## How it works

The user types a research topic in the TUI and presses Enter. The coordinator
agent drives the pipeline by calling three tools in sequence, each visible as
a live tool execution in the TUI:

```
User: "The impact of Rust on systems programming"
        │
        ▼  tool call 1
┌────────────────────────┐
│  orchestrate_research  │  → returns 3 numbered sub-questions
└────────────────────────┘
        │
        ▼  tool call 2 (×3, one per question)
┌────────────────────────┐
│   research_question    │  → runs researcher agent with SaveArtifactTool
│                        │    saves findings to research/q1.md … q3.md
└────────────────────────┘
        │
        ▼  tool call 3
┌────────────────────────┐
│  synthesize_and_save   │  → runs synthesizer with artifact_tools()
│                        │    list → load → save to report/final.md
│                        │    writes file to disk, returns path
└────────────────────────┘
        │
        ▼
"Report saved to `report.md` (4231 bytes, artifact v1)."
```

All sub-agents share one `Arc<InMemoryArtifactStore>` and a fixed `SESSION_ID`,
so every artifact saved by a researcher is immediately visible to the
synthesizer.

## Run

```bash
cargo run
```

Type a research topic and press **Enter**.  The pipeline runs automatically
and the TUI shows each tool call as it executes.  When finished, the
coordinator tells you where the report was saved.

## Key bindings

| Key | Action |
|---|---|
| `Enter` | Send message |
| `Ctrl+C` / `:q` | Quit |
| `Ctrl+Z` | Abort running generation |
| `↑ / ↓` | Scroll conversation |

## Required environment variables

| Variable | Description |
|---|---|
| `ANTHROPIC_API_KEY` | Anthropic API key |
