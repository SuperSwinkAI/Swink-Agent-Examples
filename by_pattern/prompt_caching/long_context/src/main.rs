//! Example 13: Prompt caching with a long system prompt.
//!
//! Demonstrates:
//! - `CacheConfig` to enable provider-side context caching
//! - `AgentEvent::CacheAction` reporting cache read/write events
//! - Sending 5 sequential prompts with the same large system prompt
//! - Printing per-turn token usage to show cache read savings
//!
//! Prompt caching is Anthropic-specific and requires the `anthropic` adapter.
//! The system prompt must exceed `min_tokens` (1 024) for caching to activate.
//!
//! # Run
//!
//! ```text
//! cargo run -p prompt-caching-long-context
//! ```
//!
//! # Requires
//!
//! - `ANTHROPIC_API_KEY`

use swink_agent::CacheConfig;
use swink_agent::AgentEvent;
use swink_agent::prelude::*;
use swink_agent_adapters::build_remote_connection_for_model;

// ─── Large system prompt (~500 words) ────────────────────────────────────────

const SYSTEM_PROMPT: &str = "\
You are an expert Rust programming tutor. Below is a comprehensive reference \
on core Rust concepts that you should use to answer all questions precisely.

## Ownership

Rust's ownership system is its most distinctive feature. Every value in Rust \
has a single owner — the variable that holds it. When the owner goes out of \
scope the value is dropped and its memory is freed automatically. This \
eliminates entire classes of bugs like use-after-free, double-free, and \
memory leaks without needing a garbage collector.

Ownership can be transferred (moved) to another binding, after which the \
original binding is no longer valid. Types that implement the Copy trait — \
integers, booleans, chars, and tuples of Copy types — are duplicated instead \
of moved.

## Borrowing and References

Instead of transferring ownership you can lend a reference. Shared references \
(&T) allow multiple simultaneous readers but no writers. Mutable references \
(&mut T) allow exactly one writer with no simultaneous readers. The compiler \
enforces these rules at compile time through the borrow checker, preventing \
data races entirely.

References must never outlive the data they point to. The compiler tracks \
lifetimes — the scopes during which references remain valid — and rejects \
programs where a reference could dangle.

## Lifetimes

Lifetimes are Rust's way of expressing how long references are valid. They \
are usually inferred by the compiler but must occasionally be written \
explicitly using annotation syntax: 'a, 'b, and so on. Lifetime annotations \
describe relationships between input and output references rather than \
specifying concrete durations. The 'static lifetime denotes data that lives \
for the entire program — string literals and static variables.

## Traits

Traits define shared behaviour. A trait declares a set of methods; types \
implement it by providing concrete definitions. Traits enable polymorphism: \
a function that accepts impl Trait works with any type implementing that \
trait. Trait objects (dyn Trait) enable runtime dispatch when the concrete \
type is unknown at compile time. The standard library ships dozens of core \
traits: Display, Debug, Clone, Copy, Iterator, From, Into, Error, and more.

## Async / Await

Rust's async / await syntax enables writing concurrent code that looks \
synchronous. An async fn returns a Future — a value representing a \
computation that may not have completed yet. Futures are lazy: they only \
make progress when polled by an executor such as Tokio or async-std. The \
.await operator suspends the current async function until the future resolves, \
yielding control back to the executor so other tasks can run. This cooperative \
multitasking model achieves high concurrency without OS threads for every task.

## Error Handling

Rust uses the Result<T, E> type instead of exceptions. Functions that can \
fail return Result; callers propagate errors with the ? operator, which \
returns early on Err. The Error trait is the standard interface for error \
types; the thiserror and anyhow crates simplify implementing and working with \
errors respectively.

## Generics

Generic functions and structs are parameterised over types, enabling reuse \
without runtime overhead. The compiler monomorphises generics, generating \
specialised machine code for each concrete type used. Trait bounds constrain \
which types a generic parameter accepts: fn largest<T: PartialOrd>(list: &[T]) \
accepts any type that supports comparison.

Answer all questions using the material above. Be precise and concise.";

// ─── Main ────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let connection = build_remote_connection_for_model("claude-haiku-4-5-20251001")?;
    let connections = ModelConnections::new(connection, vec![]);

    // Enable prompt caching: TTL 5 min, min 1 024 tokens, refresh every 4 turns.
    let options = AgentOptions::from_connections(SYSTEM_PROMPT, connections)
        .with_cache_config(CacheConfig::new(
            std::time::Duration::from_secs(300),
            1024,
            4,
        ))
        .with_event_forwarder(|event| {
            if let AgentEvent::CacheAction { hint, prefix_tokens } = event {
                println!("[cache] action={hint:?} prefix_tokens={prefix_tokens}");
            }
        });

    let mut agent = Agent::new(options);

    let prompts = [
        "What is ownership in Rust?",
        "Explain lifetimes.",
        "How does async/await work in Rust?",
        "What are trait objects?",
        "Summarize what we have covered.",
    ];

    let mut total_input = 0u64;
    let mut total_output = 0u64;

    for (i, prompt) in prompts.iter().copied().enumerate() {
        println!("\n--- Turn {} ---", i + 1);
        println!(">>> {prompt}");
        let result = agent.prompt_text(prompt).await?;
        println!("Assistant: {}", result.assistant_text());
        println!(
            "[usage] input={} output={} total={}",
            result.usage.input, result.usage.output, result.usage.total
        );
        total_input += result.usage.input;
        total_output += result.usage.output;
    }

    println!("\n--- Summary ---");
    println!("Total input tokens:  {total_input}");
    println!("Total output tokens: {total_output}");
    println!(
        "After the first turn, subsequent turns should show [cache] action=Read events above, \
         indicating the system prompt was served from Anthropic's cache."
    );

    Ok(())
}
