# local/smollm3/basic

Runs a single-turn agent loop using SmolLM3-3B entirely on-device.

No API key required. The model (~1.92 GB, GGUF Q4_K_M quantization) is downloaded from Hugging Face on first run and cached locally. Subsequent runs load from cache.

## Known issue

`swink-agent-local-llm` 0.7.7 depends on `mistralrs` 0.8.1, which does not yet recognise the `smollm3` GGUF architecture. Running this example currently panics after the model downloads. Tracked in SuperSwinkAI/Swink-Agent#586 — update `swink-agent-local-llm` once a fix is released.

## Run

```bash
# CPU (works on any machine)
cargo run

# Apple Silicon GPU (faster)
cargo run --features metal

# NVIDIA GPU (faster)
cargo run --features cuda
```

## What it demonstrates

- `default_local_connection()` — zero-config entry point for SmolLM3-3B
- `swink-agent-local-llm` as a drop-in `ModelConnection` with no provider setup
- Hardware backend selection via feature flags (`metal`, `cuda`)

## Model details

| Property | Value |
|---|---|
| Model | SmolLM3-3B |
| Quantization | Q4_K_M |
| Size | ~1.92 GB |
| Context window | 8192 tokens |
| CPU inference | Supported |
| Source | Hugging Face (auto-downloaded) |

## Overriding the model

The model repo and filename can be swapped at runtime via environment variables — no recompile needed:

```bash
LOCAL_MODEL_REPO=HuggingFaceTB/SmolLM3-3B-Instruct-GGUF \
LOCAL_MODEL_FILE=SmolLM3-3B-Instruct-Q8_0.gguf \
cargo run
```
