//! Minimal example: create an Agent with a mock stream function, send a prompt,
//! and print the result.
//!
//! This demonstrates the core API without any real LLM provider. In production
//! you would replace `MockStreamFn` with an adapter from `swink-agent-adapters`
//! (e.g. `AnthropicStreamFn`, `OpenAiStreamFn`, `OllamaStreamFn`).

use std::pin::Pin;
use std::sync::Mutex;

use futures::Stream;
use tokio_util::sync::CancellationToken;

use swink_agent::prelude::*;

// ─── Mock StreamFn ──────────────────────────────────────────────────────────

/// A mock `StreamFn` that returns a canned text response.
struct MockStreamFn {
    responses: Mutex<Vec<Vec<AssistantMessageEvent>>>,
}

impl MockStreamFn {
    fn new(responses: Vec<Vec<AssistantMessageEvent>>) -> Self {
        Self {
            responses: Mutex::new(responses),
        }
    }
}

impl StreamFn for MockStreamFn {
    fn stream<'a>(
        &'a self,
        _model: &'a ModelSpec,
        _context: &'a swink_agent::AgentContext,
        _options: &'a StreamOptions,
        _cancellation_token: CancellationToken,
    ) -> Pin<Box<dyn Stream<Item = AssistantMessageEvent> + Send + 'a>> {
        let events = {
            let mut responses = self.responses.lock().unwrap();
            if responses.is_empty() {
                vec![AssistantMessageEvent::Error {
                    stop_reason: StopReason::Error,
                    error_message: "no more scripted responses".to_string(),
                    error_kind: None,
                    usage: None,
                }]
            } else {
                responses.remove(0)
            }
        };
        Box::pin(futures::stream::iter(events))
    }
}

// ─── Main ───────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    // Step 1: Create a mock stream function with a canned response.
    let stream_fn = std::sync::Arc::new(MockStreamFn::new(vec![
        AssistantMessageEvent::text_response(
            "To cover the empty-queue path: script a test where you exhaust all responses \
             then verify the agent surfaces the error instead of panicking.",
        ),
        AssistantMessageEvent::text_response(
            "For rate-limit retry testing: add an Error event with error_kind set to \
             RateLimit, then assert your retry wrapper fires before the next scripted response.",
        ),
    ]));

    // Step 2: Define the model specification.
    let model = ModelSpec::new("mock", "mock-model-v1");

    // Step 3: Build agent options with defaults.
    let options = AgentOptions::new_simple("You are a helpful assistant.", model, stream_fn);

    // Step 4: Create the agent.
    let mut agent = Agent::new(options);

    // Step 5: Send prompts and print results.
    for prompt in [
        "I've got a MockStreamFn with scripted responses. What test cases should I add to validate that the agent handles an empty response queue gracefully?",
        "My production StreamFn occasionally returns rate-limit errors. How do I script that in the mock to exercise my retry logic?",
    ] {
        println!(">>> {prompt}");
        let result = agent.prompt_text(prompt).await.expect("prompt failed");
        println!("Assistant: {}", result.assistant_text());
        println!("Stop reason: {:?}", result.stop_reason);
        println!("Usage: {:?}\n", result.usage);
    }
}
