---
type: source
title: drive-research-token-efficient-bol-processing-architecture
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# drive-research-token-efficient-bol-processing-architecture

Ingested source summary (2026-06-08).

## Entities
- [[layoutlmv3|LayoutLMv3]] (TOOL)
- [[large-language-models-llms|Large Language Models (LLMs)]] (SYSTEM)
- [[portkey|Portkey]] (TOOL)
- [[gguf-gpt-generated-unified-format|GGUF (GPT-Generated Unified Format)]] (CONCEPT)
- [[named-entity-recognition-ner|Named Entity Recognition (NER)]] (CONCEPT)
- [[slashllm|SlashLLM]] (TOOL)
- [[llamacpp-binding|LlamaCpp binding]] (TOOL)
- [[docling|Docling]] (TOOL)
- [[heron|Heron]] (SYSTEM)
- [[pydantic|Pydantic]] (TOOL)
- [[deterministic-boilerplate-stripping|Deterministic Boilerplate Stripping]] (CONCEPT)
- [[token-optimized-data-serialization-formats|Token-Optimized Data Serialization Formats]] (CONCEPT)
- [[opensearch|OpenSearch]] (TOOL)
- [[multi-stage-extraction|Multi-Stage Extraction]] (CONCEPT)
- [[llama-cpp|llama.cpp]] (TOOL)
- [[topicnodeparser|TopicNodeParser]] (TOOL)
- [[kong-ai-gateway|Kong AI Gateway]] (TOOL)
- [[context-engineering|Context Engineering]] (CONCEPT)
- [[dynamic-context-assembly|Dynamic Context Assembly]] (CONCEPT)
- [[litellm|LiteLLM]] (TOOL)
- [[longllmlinguapostprocessor|LongLLMLinguaPostprocessor]] (TOOL)
- [[documentconverter|DocumentConverter()]] (TOOL)
- [[surya|Surya]] (TOOL)
- [[mooncake|Mooncake]] (TOOL)
- [[prompt-compression|Prompt Compression]] (CONCEPT)
- [[document-structure-based-chunking|Document Structure-Based Chunking]] (CONCEPT)
- [[llmlingua-2|LLMLingua-2]] (TOOL)
- [[retrieval-augmented-generation-rag|Retrieval-Augmented Generation (RAG)]] (CONCEPT)
- [[d-j-vu|DéjàVu]] (TOOL)
- [[easyocr|EasyOCR]] (TOOL)
- [[doclingdocument|DoclingDocument]] (SYSTEM)
- [[llamaindex-workflows|LlamaIndex Workflows]] (TOOL)
- [[bills-of-lading-bols|Bills of Lading (BoLs)]] (CONCEPT)
- [[hybridserve|HybridServe]] (TOOL)
- [[semantic-chunking|Semantic Chunking]] (CONCEPT)
- [[semanticsplitternodeparser|SemanticSplitterNodeParser]] (TOOL)
- [[automatic-prefix-caching-apc|Automatic Prefix Caching (APC)]] (CONCEPT)
- [[langchain|LangChain]] (TOOL)
- [[llamaparse|LlamaParse]] (TOOL)
- [[mastra|Mastra]] (TOOL)
- [[tesseract|Tesseract]] (TOOL)
- [[quantization|Quantization]] (CONCEPT)
- [[visual-document-understanding-vdu|Visual Document Understanding (VDU)]] (CONCEPT)
- [[document-understanding-transformer-donut|Document Understanding Transformer (Donut)]] (TOOL)
- [[langgraph|LangGraph]] (TOOL)
- [[dycp-dynamic-context-management|DyCP (Dynamic Context Management)]] (CONCEPT)
- [[longllmlingua|LongLLMLingua]] (TOOL)
- [[model-context-protocols-mcp|Model Context Protocols (MCP)]] (CONCEPT)
- [[vllm|vLLM]] (TOOL)
- [[milvus|Milvus]] (TOOL)
- [[semantic-caching|semantic caching]] (CONCEPT)
- [[bill-of-lading-bol|Bill of Lading (BoL)]] (CONCEPT)
- [[hybrid-queries|Hybrid Queries]] (CONCEPT)
- [[kv-cache|KV cache]] (CONCEPT)
- [[reciprocal-rank-fusion-rrf|Reciprocal Rank Fusion (RRF)]] (CONCEPT)
- [[orchestration-frameworks|Orchestration Frameworks]] (CONCEPT)
- [[agent-to-agent-a2a-governance|Agent-to-Agent (A2A) Governance]] (CONCEPT)
- [[optical-character-recognition-ocr|Optical Character Recognition (OCR)]] (TOOL)
- [[redis|Redis]] (TOOL)

## Relations
- LLMs → USES → Optical Character Recognition (OCR)
- Visual Document Understanding (VDU) → RELATED_TO → LLMs
- Visual Document Understanding (VDU) → RELATED_TO → Optical Character Recognition (OCR)
- LayoutLMv3 → RELATED_TO → Visual Document Understanding (VDU)
- Document Understanding Transformer (Donut) → RELATED_TO → Visual Document Understanding (VDU)
- Surya → RELATED_TO → Visual Document Understanding (VDU)
- Docling → USES → Heron
- Heron → RELATED_TO → DoclingDocument
- DoclingDocument → PART_OF → Pydantic
- Docling → USES → DocumentConverter()
- Deterministic Boilerplate Stripping → RELATED_TO → Visual Document Understanding (VDU)
- Token-Optimized Data Serialization Formats → RELATED_TO → LLMs
- Multi-Stage Extraction → USES → Named Entity Recognition (NER)
- Multi-Stage Extraction → USES → LLMs
- Named Entity Recognition (NER) → USES → LlamaCpp binding
- LlamaCpp binding → USES → LangChain
- llama.cpp → USES → GGUF (GPT-Generated Unified Format)
- Quantization → RELATED_TO → llama.cpp
- Context Engineering → RELATED_TO → Prompt Compression
- Context Engineering → RELATED_TO → Dynamic Context Assembly
- Prompt Compression → USES → LLMLingua-2
- LLMLingua-2 → RELATED_TO → Prompt Compression
- LongLLMLingua → RELATED_TO → Retrieval-Augmented Generation (RAG)
- LongLLMLinguaPostprocessor → USES → LlamaIndex Workflows
- Dynamic Context Assembly → USES → Model Context Protocols (MCP)
- DyCP (Dynamic Context Management) → RELATED_TO → Dynamic Context Assembly
- Retrieval-Augmented Generation (RAG) → RELATED_TO → LlamaIndex Workflows
- Semantic Chunking → RELATED_TO → Retrieval-Augmented Generation (RAG)
- Document Structure-Based Chunking → USES → Visual Document Understanding (VDU)
- Docling → RELATED_TO → Document Structure-Based Chunking
- OpenSearch → RELATED_TO → Hybrid Queries
- OpenSearch → USES → Document Structure-Based Chunking
- Milvus → RELATED_TO → LlamaIndex Workflows
- Hybrid Queries → USES → Reciprocal Rank Fusion (RRF)
- Semantic Chunking → USES → SemanticSplitterNodeParser
- Orchestration Frameworks → RELATED_TO → LangGraph
- Orchestration Frameworks → RELATED_TO → LlamaIndex Workflows
- Orchestration Frameworks → RELATED_TO → Mastra
- LangGraph → PART_OF → Orchestration Frameworks
- LlamaIndex Workflows → PART_OF → Orchestration Frameworks
- Mastra → PART_OF → Orchestration Frameworks
- LlamaIndex Workflows → USES → Docling
- Kong AI Gateway → RELATED_TO → LLMs
- Kong AI Gateway → RELATED_TO → Orchestration Frameworks
- Kong AI Gateway → USES → Redis
- Kong AI Gateway → PART_OF → Bill of Lading (BoL)
- Kong AI Gateway → USES → semantic caching
- Kong AI Gateway → USES → Agent-to-Agent (A2A) Governance
- LiteLLM → USES → Redis
- LiteLLM → PART_OF → Bill of Lading (BoL)
- Portkey → USES → semantic caching
- SlashLLM → PART_OF → Bill of Lading (BoL)
- vLLM → USES → KV cache
- vLLM → USES → Automatic Prefix Caching (APC)
- vLLM → PART_OF → Bill of Lading (BoL)
- DéjàVu → USES → KV cache
- HybridServe → USES → KV cache
- Mooncake → USES → KV cache
- LangGraph → RELATED_TO → Agent-to-Agent (A2A) Governance
- LlamaIndex Workflows → RELATED_TO → Agent-to-Agent (A2A) Governance
- Bill of Lading (BoL) → RELATED_TO → Prompt Compression
- Bill of Lading (BoL) → RELATED_TO → Document Structure-Based Chunking
