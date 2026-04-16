//! Example: local inference with SmolLM3-3B.
//!
//! Runs a single-turn agent loop entirely on-device — no API key, no network
//! call after the first run. The model (~1.92 GB GGUF) is downloaded from
//! Hugging Face on first use and cached locally by `mistralrs`.
//!
//! SmolLM3-3B works on CPU. For faster inference on Apple Silicon compile
//! with `--features metal`; on NVIDIA compile with `--features cuda`.
//!
//! # Run
//!
//! ```text
//! cargo run                        # CPU
//! cargo run --features metal       # Apple Silicon GPU
//! cargo run --features cuda        # NVIDIA GPU
//! ```

use swink_agent::{AgentOptions, ModelConnections, RunOptions};
use swink_agent_local_llm::default_local_connection;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let connection = default_local_connection()?;
    let connections = ModelConnections::new(connection, vec![]);

    let options = AgentOptions::from_connections(
        "You are a helpful assistant running entirely on-device.",
        connections,
    );

    let result = options
        .run(
            "In one sentence, what is the capital of France?",
            RunOptions::default(),
        )
        .await?;

    println!("{}", result.content);

    Ok(())
}
