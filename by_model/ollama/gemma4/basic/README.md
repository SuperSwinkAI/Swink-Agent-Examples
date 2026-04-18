# Ollama Gemma 4 — Basic

Minimal single-turn prompt using a local Ollama instance with Gemma 4 31B.

## Requirements

- [Ollama](https://ollama.com) running locally (`ollama serve`)
- Model pulled: `ollama pull gemma4:31b`
- GPU with sufficient VRAM (~20 GB for 31B); Ollama uses the GPU automatically

## GPU Setup

Ollama detects and uses your GPU automatically. To ensure all layers offload to
the GPU and enable flash attention for better VRAM efficiency, set these before
starting Ollama:

**Windows (PowerShell):**
```powershell
$env:OLLAMA_NUM_GPU = -1
$env:OLLAMA_FLASH_ATTENTION = 1
ollama serve
```

**Linux/macOS:**
```bash
OLLAMA_NUM_GPU=-1 OLLAMA_FLASH_ATTENTION=1 ollama serve
```

## Run

```bash
cargo run
```
