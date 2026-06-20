---
type: source
title: ultimate-local-ai-development-stack-for-vscodium-micro02
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# ultimate-local-ai-development-stack-for-vscodium-micro02

Ingested source summary (2026-06-09).

## Entities
- [RAG (Retrieval-Augmented Generation)](/entities/rag-retrieval-augmented-generation.md) (CONCEPT)
- [OpenClaw](/entities/openclaw.md) (TOOL)
- [The Ultimate FOSS Local AI Setup](/entities/the-ultimate-foss-local-ai-setup.md) (PROJECT)
- [Assisted Generation](/entities/assisted-generation.md) (CONCEPT)
- [LM Studio](/entities/lm-studio.md) (TOOL)
- [DeepSeek-V3](/entities/deepseek-v3.md) (MODEL)
- [@modelcontextprotocol/server-postgres](/entities/modelcontextprotocol-server-postgres.md) (TOOL)
- [Qwen-2.5-Coder-0.5B-Instruct](/entities/qwen-2-5-coder-0-5b-instruct.md) (MODEL)
- [Qwen-2.5-Coder-14B-Instruct](/entities/qwen-2-5-coder-14b-instruct.md) (MODEL)
- [@modelcontextprotocol/server-github](/entities/modelcontextprotocol-server-github.md) (TOOL)
- [Model Context Protocol (MCP)](/entities/model-context-protocol-mcp.md) (CONCEPT)
- [EAGLE](/entities/eagle.md) (CONCEPT)
- [Qwen-2.5-Coder-7B-Instruct](/entities/qwen-2-5-coder-7b-instruct.md) (MODEL)
- [@modelcontextprotocol/server-puppeteer](/entities/modelcontextprotocol-server-puppeteer.md) (TOOL)
- [Llama-3.1-8B-Instruct](/entities/llama-3-1-8b-instruct.md) (MODEL)
- [DeepSeek-R1-Distill-Qwen-14B](/entities/deepseek-r1-distill-qwen-14b.md) (MODEL)
- [DeepSeek-R1-Distill-Qwen-1.5B](/entities/deepseek-r1-distill-qwen-1-5b.md) (MODEL)
- [Qwen-2.5-Coder-32B-Instruct](/entities/qwen-2-5-coder-32b-instruct.md) (MODEL)
- [Prompt Lookup Decoding](/entities/prompt-lookup-decoding.md) (CONCEPT)
- [VSCodium](/entities/vscodium.md) (TOOL)
- [Aider](/entities/aider.md) (TOOL)
- [Medusa Architectures](/entities/medusa-architectures.md) (CONCEPT)
- [Qwen-2.5-Coder-1.5B](/entities/qwen-2-5-coder-1-5b.md) (MODEL)
- [Speculative Decoding](/entities/speculative-decoding.md) (CONCEPT)
- [Draft-and-Verify](/entities/draft-and-verify.md) (CONCEPT)
- [DevContainers](/entities/devcontainers.md) (CONCEPT)
- [KV Context Caching](/entities/kv-context-caching.md) (CONCEPT)
- [Continue.dev](/entities/continue-dev.md) (TOOL)
- [Llama-3.2-1B-Instruct](/entities/llama-3-2-1b-instruct.md) (MODEL)
- [Roo Code](/entities/roo-code.md) (TOOL)
- [@modelcontextprotocol/server-sqlite](/entities/modelcontextprotocol-server-sqlite.md) (TOOL)
- [nomic-embed-text-v1.5](/entities/nomic-embed-text-v1-5.md) (MODEL)
- [@modelcontextprotocol/server-fetch](/entities/modelcontextprotocol-server-fetch.md) (TOOL)
- [Multi-Token Prediction (MTP)](/entities/multi-token-prediction-mtp.md) (CONCEPT)

## Relations
- Continue.dev → USES → Qwen-2.5-Coder-1.5B
- Continue.dev → USES → LM Studio
- Roo Code → USES → DeepSeek-R1-Distill-Qwen-14B
- Roo Code → USES → Qwen-2.5-Coder-32B-Instruct
- Roo Code → USES → LM Studio
- The Ultimate FOSS Local AI Setup → PART_OF → Continue.dev
- The Ultimate FOSS Local AI Setup → PART_OF → Roo Code
- The Ultimate FOSS Local AI Setup → PART_OF → LM Studio
- The Ultimate FOSS Local AI Setup → PART_OF → VSCodium
- Speculative Decoding → RELATED_TO → Draft-and-Verify
- Speculative Decoding → RELATED_TO → Assisted Generation
- Speculative Decoding → RELATED_TO → Multi-Token Prediction (MTP)
- Speculative Decoding → RELATED_TO → EAGLE
- Speculative Decoding → RELATED_TO → Medusa Architectures
- Speculative Decoding → RELATED_TO → Prompt Lookup Decoding
- LM Studio → USES → Speculative Decoding
- Multi-Token Prediction (MTP) → RELATED_TO → DeepSeek-V3
- Multi-Token Prediction (MTP) → RELATED_TO → DeepSeek-R1-Distill-Qwen-14B
- EAGLE → RELATED_TO → Medusa Architectures
- LM Studio → USES → Qwen-2.5-Coder-0.5B-Instruct
- LM Studio → USES → Qwen-2.5-Coder-14B-Instruct
- LM Studio → USES → Llama-3.1-8B-Instruct
- LM Studio → USES → Llama-3.2-1B-Instruct
- LM Studio → USES → DeepSeek-R1-Distill-Qwen-14B
- LM Studio → USES → DeepSeek-R1-Distill-Qwen-1.5B
- Model Context Protocol (MCP) → RELATED_TO → Roo Code
- Roo Code → USES → @modelcontextprotocol/server-sqlite
- Roo Code → USES → @modelcontextprotocol/server-postgres
- Roo Code → USES → @modelcontextprotocol/server-fetch
- Roo Code → USES → @modelcontextprotocol/server-puppeteer
- Roo Code → USES → @modelcontextprotocol/server-github
- KV Context Caching → USES → LM Studio
- RAG (Retrieval-Augmented Generation) → USES → Continue.dev
- Continue.dev → USES → nomic-embed-text-v1.5
- LM Studio → USES → nomic-embed-text-v1.5
- Continue.dev → USES → VSCodium
- Roo Code → USES → VSCodium
- DevContainers → RELATED_TO → Roo Code
