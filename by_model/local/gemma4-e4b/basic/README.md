# local/gemma4-e4b/basic

Run a multi-turn conversation entirely on-device using Gemma 4 E4B (4-billion
effective-parameter model from Google).

## What it does

Sends two prompts to a local Gemma 4 E4B agent and prints each reply. The
second prompt refers back to the first answer, demonstrating that context is
preserved across turns.

## How it works

Gemma 4 models require explicit connection construction — look up the preset
from the catalog, create a `LocalModel` from the matching `ModelPreset`, then
wire them together with `LocalStreamFn`:

```rust
let preset = model_catalog()
    .preset("local", "gemma4_e4b")
    .ok_or("gemma4_e4b not found in catalog")?;
let model_spec = preset.model_spec();
let model = LocalModel::from_preset(ModelPreset::Gemma4E4B);
let connection = ModelConnection::new(
    model_spec,
    Arc::new(LocalStreamFn::new(Arc::new(model))),
);
```

The model file is downloaded from Hugging Face on first run and cached at
`~/.cache/huggingface/hub/`.

The `gemma4` feature flag on `swink-agent-local-llm` must be enabled (see
`Cargo.toml`) — it pulls in the tokenizer and prompt-formatting support
required by the Gemma 4 architecture.

## Model

| Field | Value |
|---|---|
| Model | bartowski/google_gemma-4-E4B-it-GGUF |
| Quant | Q4_K_M |
| Download size | ~5.5 GB |
| Context | 131 072 tokens |

## Running

```bash
cargo run
```

Enable Metal (macOS) or CUDA acceleration:

```bash
cargo run --features metal
cargo run --features cuda
```
