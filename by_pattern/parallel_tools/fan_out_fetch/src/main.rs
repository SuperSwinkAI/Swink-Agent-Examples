//! Example: fan-out tool calls with three execution policies.
//!
//! Four simulated IO tools (weather, news, stock, exchange) each sleep 500 ms.
//! By switching `ToolExecutionPolicy` you can observe:
//!
//! - **sequential**  — tools run one at a time (~2 s total)
//! - **concurrent**  — all tools run in parallel (~0.5 s total, default)
//! - **priority**    — weather runs first (higher priority), then the rest
//!
//! Usage:
//!   cargo run -- --mode sequential
//!   cargo run -- --mode concurrent
//!   cargo run -- --mode priority

use std::sync::Arc;

use schemars::JsonSchema;
use serde::Deserialize;
use swink_agent::{
    Agent, AgentOptions, AgentToolResult, FnTool, IntoTool, ModelConnection, ModelConnections,
    ToolExecutionPolicy,
};
use swink_agent_adapters::build_remote_connection_for_model;
use tokio::time::{Duration, sleep};

// ── Tool parameter types ────────────────────────────────────────────────────

#[derive(Deserialize, JsonSchema)]
struct CityParams {
    /// The city to fetch weather for (e.g. "London").
    city: String,
}

#[derive(Deserialize, JsonSchema)]
struct TopicParams {
    /// The news topic to search (e.g. "technology").
    topic: String,
}

#[derive(Deserialize, JsonSchema)]
struct TickerParams {
    /// The stock ticker symbol (e.g. "AAPL").
    ticker: String,
}

#[derive(Deserialize, JsonSchema)]
struct PairParams {
    /// The currency pair (e.g. "EUR/USD").
    pair: String,
}

// ── Tool builders ───────────────────────────────────────────────────────────

fn fetch_weather_tool() -> Arc<dyn swink_agent::AgentTool> {
    FnTool::new(
        "fetch_weather",
        "Weather",
        "Fetch the current weather for a city.",
    )
    .with_execute_typed::<CityParams, _, _>(|params, _cancel| async move {
        sleep(Duration::from_millis(500)).await;
        AgentToolResult::text(format!(
            "Weather in {}: 18°C, partly cloudy, humidity 65%.",
            params.city
        ))
    })
    .into_tool()
}

fn fetch_news_tool() -> Arc<dyn swink_agent::AgentTool> {
    FnTool::new(
        "fetch_news",
        "News",
        "Fetch the top headlines for a topic.",
    )
    .with_execute_typed::<TopicParams, _, _>(|params, _cancel| async move {
        sleep(Duration::from_millis(500)).await;
        AgentToolResult::text(format!(
            "Top {} headlines: (1) Rust 2.0 roadmap published. \
             (2) AI chip shortage eases. (3) Open-source LLM beats GPT-4 on benchmarks.",
            params.topic
        ))
    })
    .into_tool()
}

fn fetch_stock_tool() -> Arc<dyn swink_agent::AgentTool> {
    FnTool::new(
        "fetch_stock",
        "Stock",
        "Fetch the current price for a stock ticker.",
    )
    .with_execute_typed::<TickerParams, _, _>(|params, _cancel| async move {
        sleep(Duration::from_millis(500)).await;
        AgentToolResult::text(format!(
            "{} is trading at $189.42 (+1.3% today).",
            params.ticker
        ))
    })
    .into_tool()
}

fn fetch_exchange_tool() -> Arc<dyn swink_agent::AgentTool> {
    FnTool::new(
        "fetch_exchange",
        "Exchange",
        "Fetch the exchange rate for a currency pair.",
    )
    .with_execute_typed::<PairParams, _, _>(|params, _cancel| async move {
        sleep(Duration::from_millis(500)).await;
        AgentToolResult::text(format!("{} = 1.0821 (mid-market rate).", params.pair))
    })
    .into_tool()
}

// ── Mode parsing ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
enum Mode {
    Sequential,
    Concurrent,
    Priority,
}

fn parse_mode(args: &[String]) -> Mode {
    let mode_str = args
        .windows(2)
        .find(|w| w[0] == "--mode")
        .map(|w| w[1].as_str())
        .unwrap_or("concurrent");

    match mode_str {
        "sequential" => Mode::Sequential,
        "priority" => Mode::Priority,
        _ => Mode::Concurrent,
    }
}

// ── Run agent with a given policy ───────────────────────────────────────────

async fn run_with_policy(
    connection: ModelConnection,
    policy: ToolExecutionPolicy,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let tools = vec![
        fetch_weather_tool(),
        fetch_news_tool(),
        fetch_stock_tool(),
        fetch_exchange_tool(),
    ];

    let options = AgentOptions::from_connections(
        "You are an information aggregator. When asked for a briefing, use ALL available \
         tools to gather data, then synthesise a concise summary.",
        ModelConnections::new(connection, vec![]),
    )
    .with_tools(tools)
    .with_tool_execution_policy(policy);

    let mut agent = Agent::new(options);

    let prompt = "Give me a morning briefing. Check the weather in London, the top tech \
                  news headlines, the AAPL stock price, and the EUR/USD exchange rate.";

    let start = tokio::time::Instant::now();
    let result = agent.prompt_text(prompt).await?;
    let elapsed = start.elapsed();

    println!("=== {label} mode (elapsed: {elapsed:.2?}) ===");
    println!("{}\n", result.assistant_text());

    Ok(())
}

// ── Main ────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let args: Vec<String> = std::env::args().collect();
    let mode = parse_mode(&args);

    println!("Running in {mode:?} mode\n");

    let connection = build_remote_connection_for_model("claude-haiku-4-5-20251001")?;

    let policy = match mode {
        Mode::Sequential => ToolExecutionPolicy::Sequential,
        Mode::Concurrent => ToolExecutionPolicy::Concurrent,
        Mode::Priority => {
            // fetch_weather gets priority 10; all other tools get priority 5.
            // Tools within the same priority group still execute concurrently.
            ToolExecutionPolicy::Priority(Arc::new(|call| {
                if call.name == "fetch_weather" { 10 } else { 5 }
            }))
        }
    };

    let label = format!("{mode:?}");
    run_with_policy(connection, policy, &label).await?;

    Ok(())
}
