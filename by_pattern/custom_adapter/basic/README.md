# custom_adapter/basic

Shows how to implement the `StreamFn` trait to connect any LLM backend — whether a local HTTP service, a proprietary API, or a test double.

## What it shows

- The full `StreamFn` contract: `stream(&self, model, context, options, cancellation_token) -> Pin<Box<dyn Stream<Item = AssistantMessageEvent>>>`
- The required event sequence: `Start` → `TextStart` → `TextDelta` (one or more) → `TextEnd` → `Done`
- Reading the conversation context (`context.messages`) inside the stream function
- Wiring the custom adapter into `AgentOptions::new_simple`

## Run

```bash
cargo run
```

No API key required — the `DummyStreamFn` echoes the user's message back.

## Building a real adapter

A real HTTP adapter follows the same shape:

```rust
impl StreamFn for MyProviderStreamFn {
    fn stream<'a>(&'a self, model: &'a ModelSpec, context: &'a AgentContext, ...) -> ... {
        Box::pin(async_stream::stream! {
            yield AssistantMessageEvent::Start;
            // make HTTP request, parse SSE or NDJSON chunks ...
            // yield TextStart / TextDelta / TextEnd / ToolCallStart / ... events
            yield AssistantMessageEvent::Done { ... };
        })
    }
}
```

See the built-in adapters in `swink-agent-adapters` for complete reference implementations.
