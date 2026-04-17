# local/smollm3/basic

Run a multi-turn conversation entirely on-device using SmolLM3-3B.

## What it does

Sends two prompts to a local SmolLM3 agent and prints each reply. The second
prompt refers back to the first answer, demonstrating that context is preserved
across turns.

## How it works

`default_local_connection()` handles everything: it looks up the SmolLM3 preset
in the model catalog, constructs the `LocalModel`, and returns a ready-to-use
`ModelConnection`. The model file is downloaded from Hugging Face on first run
and cached at `~/.cache/huggingface/hub/`.

```rust
let connection = default_local_connection()?;
let connections = ModelConnections::new(connection, vec![]);
let options = AgentOptions::from_connections("You are a helpful assistant...", connections);
let mut agent = Agent::new(options);
```

## Model

| Field | Value |
|---|---|
| Model | HuggingFaceTB/SmolLM3-3B |
| Quant | Q4_K_M |
| Download size | ~1.9 GB |
| Context | 8 192 tokens |

## Running

```bash
cargo run
```

Enable Metal (macOS) or CUDA acceleration:

```bash
cargo run --features metal
cargo run --features cuda
```

### Note on Metal coverage

`--features metal` compiles and loads the Metal backend, but SmolLM3's
architecture uses a fused Gated Delta Net attention variant that does not yet
have Metal kernels in the bundled llama.cpp. Runtime logs will show the KV
cache allocated on CPU:

```text
llama_kv_cache:        CPU KV buffer size =   576.00 MiB
```

This is expected — the model still runs end-to-end, just without KV-cache
acceleration on Metal. The Gemma 4 examples under `by_model/local/gemma4-*`
are fully Metal-accelerated if you need that.
