//! Implement StreamFn with a canned response following the start/delta/end protocol.

use std::pin::Pin;
use std::sync::Arc;

use futures::Stream;
use tokio_util::sync::CancellationToken;

use swink_agent::prelude::*;

/// A StreamFn that echoes the last user message.
struct DummyStreamFn;

impl StreamFn for DummyStreamFn {
    fn stream<'a>(
        &'a self,
        _model: &'a ModelSpec,
        context: &'a AgentContext,
        _options: &'a StreamOptions,
        _cancellation_token: CancellationToken,
    ) -> Pin<Box<dyn Stream<Item = AssistantMessageEvent> + Send + 'a>> {
        let user_text = context
            .messages
            .iter()
            .rev()
            .find_map(|msg| match msg {
                AgentMessage::Llm(LlmMessage::User(u)) => {
                    Some(ContentBlock::extract_text(&u.content))
                }
                _ => None,
            })
            .unwrap_or_else(|| "...".to_string());

        let response = format!("Echo: {user_text}");

        let events = vec![
            AssistantMessageEvent::Start,
            AssistantMessageEvent::TextStart { content_index: 0 },
            AssistantMessageEvent::TextDelta {
                content_index: 0,
                delta: response,
            },
            AssistantMessageEvent::TextEnd { content_index: 0 },
            AssistantMessageEvent::Done {
                stop_reason: StopReason::Stop,
                usage: Usage::default()
                    .with_input(10)
                    .with_output(5)
                    .with_total(15),
                cost: Cost::default(),
            },
        ];

        Box::pin(futures::stream::iter(events))
    }
}

#[tokio::main]
async fn main() {
    let stream_fn = Arc::new(DummyStreamFn);

    let model = ModelSpec::new("dummy", "echo-v1");
    let options = AgentOptions::new_simple("You are an echo bot.", model, stream_fn);
    let mut agent = Agent::new(options);

    for prompt in [
        "This DummyStreamFn just echoes messages. What's the minimum I'd need to add to make it call a real streaming HTTP API like Anthropic's instead?",
        "If the remote API returns tool_use content blocks in the stream, which AssistantMessageEvent variants would I emit, and in what order relative to the text events?",
    ] {
        println!(">>> {prompt}");
        let result = agent.prompt_text(prompt).await.expect("prompt failed");
        println!("Response: {}", result.assistant_text());
        println!("Token usage: {:?}\n", result.usage);
    }
}
