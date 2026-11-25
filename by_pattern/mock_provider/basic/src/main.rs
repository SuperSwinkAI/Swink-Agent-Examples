//! Create an Agent with a mock stream, send a prompt, and print its canned response.

use std::pin::Pin;
use std::sync::Mutex;

use futures::Stream;
use tokio_util::sync::CancellationToken;

use swink_agent::prelude::*;

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
                    retry_after: None,
                }]
            } else {
                responses.remove(0)
            }
        };
        Box::pin(futures::stream::iter(events))
    }
}

#[tokio::main]
async fn main() {
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

    let model = ModelSpec::new("mock", "mock-model-v1");

    let options = AgentOptions::new_simple("You are a helpful assistant.", model, stream_fn);

    let mut agent = Agent::new(options);

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
