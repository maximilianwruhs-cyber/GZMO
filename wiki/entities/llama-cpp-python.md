---
type: entity
title: llama-cpp-python
created: 2026-06-09
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# llama-cpp-python

Type: PROJECT

## From [[drive-research-llamacpp-optimization-blueprint-micro04|drive-research-llamacpp-optimization-blueprint-micro04]] (2026-06-09)
- LLM inference in C/C++
- Used for cloud KG extraction
- Supports speculative decoding
- Supports multi-GPU setups
- Supports batch processing mode
- Related to llama.cpp
- Has issues regarding default thread count and batched inference

## From [[resilient-rust-based-mcp-client-and-llm-orchestrat-micro06|resilient-rust-based-mcp-client-and-llm-orchestrat-micro06]] (2026-06-09)
- Highly popular library.
- Offers its own OpenAI-compatible web server.
- Server dependencies installed via pip install llama-cpp-python[server].
- Python-based server provides alternative to native llama-server.
- Routes parsing logic through Python-level dictionary manipulations.
- Uses specific chat_handlers to map OpenAI payload.
- Drops down to C API for tensor inference.
- Project with active maintenance and responsiveness.
- Core maintainers reverted default serialization behavior.
- Parser has had recent patches.
- Runtime actively compensates for metadata errors.
- Framework allows running heavily quantized models.
- Python library offers an OpenAI-compatible web server.
