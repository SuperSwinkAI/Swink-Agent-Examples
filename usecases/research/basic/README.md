# research-basic

A single-agent web research example: the agent searches the web, fetches the
most relevant page, synthesizes its findings into a Markdown report, and writes
the result to disk.

Demonstrates `swink-agent-plugin-web`'s `search` and `fetch` tools, the
built-in `write_file` tool, a custom `PostTurnPolicy` that caps the dispatch
loop at 12 turns, an `event_forwarder` that pretty-prints each tool call, and
`ApprovalMode::Bypassed` for unattended runs.

## Requires

- `ANTHROPIC_API_KEY` in environment or `.env` file (the agent uses
  `claude-sonnet-4-6`).
- An internet connection for web search and fetching.

## Run

```bash
cargo run -p research-basic -- "history of the Rust programming language"
```

The report is written to `~/research-<slug>.md`, where `<slug>` is the query
lowercased with non-alphanumeric runs collapsed to `-`. Example output path for
the command above: `~/research-history-of-the-rust-programming-language.md`.

## What to expect

Console output streams the agent's reasoning and each tool invocation:

```text
Query : history of the Rust programming language
Output: /Users/you/research-history-of-the-rust-programming-language.md

  [thinking] (1 messages)...
  [tool] search: history of the Rust programming language
  [thinking] (3 messages)...
  [tool] fetch: https://en.wikipedia.org/wiki/Rust_(programming_language)
  [tool] fetch failed: Response body exceeded configured limit of 500000 bytes ...
  [thinking] (5 messages)...
  [tool] fetch: https://www.technologyreview.com/2023/02/14/...
  [tool] write_file → /Users/you/research-history-of-the-rust-programming-language.md
  [tool] write_file ✓
```

## Notes

- `WebPlugin` is constructed with `with_max_content_length(500_000)`. Pages
  larger than 500 KB fail the fetch; the agent is expected to try an
  alternative URL. The system prompt instructs it to stop after one retry.
- The plugin's tools are extracted via `web_plugin.tools().filter(...)` so
  only `search` and `fetch` are attached. The plugin's `extract` and
  `screenshot` tools require Playwright and are not registered here.
- `with_plugin()` prefixes tool names with the plugin namespace (e.g.
  `web.search`). Anthropic's tool-use API rejects dots in tool names, so the
  tools are added by hand without the prefix.
- The `TurnLimitPolicy` (12 turns) is a hard stop to bound cost in case the
  agent enters a search-loop; normal runs complete in 3–6 turns.
