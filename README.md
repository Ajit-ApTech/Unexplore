# Unexplore Browser (with Native Uncensored Local AI)

**Private, Intelligent Agentic Browser with Integrated Native GGUF LLM Engine & Cloud AI.**

---

## Overview

Unexplore is a modern, high-performance web browser featuring:
- **Native GGUF Engine**: Run uncensored local models (Qwen 2.5, Llama 3.1, Gemma 2, Dolphin) directly on Metal GPU acceleration via `llama-server`.
- **Work & Page Assist**: Autonomous agentic workflows, tab management, web page inspection, restyling, and multi-turn lead assistance.
- **BYOK & Local Compatibility**: Connect to local LLM servers (Ollama, LM Studio, Native Engine) or your own API keys (Anthropic, OpenAI, Google, DeepSeek, Groq, Cerebras).
- **Built-in Content Blocking & Privacy**: High-speed native Rust ad-blocking and privacy shields with manual HTTP browsing support.

---

## Quick Start

### Prerequisites
- macOS (Apple Silicon recommended), Windows, or Linux
- [Rust](https://rustup.rs/) (edition 2021+)
- [Node.js](https://nodejs.org/) & [pnpm](https://pnpm.io/)
- [llama.cpp](https://github.com/ggml-org/llama.cpp) (`brew install llama.cpp`)

### Running in Development

```bash
cd zephium
pnpm dev
```

### Building for Production

```bash
cd zephium
pnpm tauri build
```

