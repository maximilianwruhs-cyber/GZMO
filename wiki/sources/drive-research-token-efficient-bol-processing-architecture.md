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
- [LayoutLMv3](/entities/layoutlmv3.md) (TOOL)
- [Large Language Models (LLMs)](/entities/large-language-models-llms.md) (SYSTEM)
- [Portkey](/entities/portkey.md) (TOOL)
- [GGUF (GPT-Generated Unified Format)](/entities/gguf-gpt-generated-unified-format.md) (CONCEPT)
- [Named Entity Recognition (NER)](/entities/named-entity-recognition-ner.md) (CONCEPT)
- [SlashLLM](/entities/slashllm.md) (TOOL)
- [LlamaCpp binding](/entities/llamacpp-binding.md) (TOOL)
- [Docling](/entities/docling.md) (TOOL)
- [Heron](/entities/heron.md) (SYSTEM)
- [Pydantic](/entities/pydantic.md) (TOOL)
- [Deterministic Boilerplate Stripping](/entities/deterministic-boilerplate-stripping.md) (CONCEPT)
- [Token-Optimized Data Serialization Formats](/entities/token-optimized-data-serialization-formats.md) (CONCEPT)
- [OpenSearch](/entities/opensearch.md) (TOOL)
- [Multi-Stage Extraction](/entities/multi-stage-extraction.md) (CONCEPT)
- [llama.cpp](/entities/llama-cpp.md) (TOOL)
- [TopicNodeParser](/entities/topicnodeparser.md) (TOOL)
- [Kong AI Gateway](/entities/kong-ai-gateway.md) (TOOL)
- [Context Engineering](/entities/context-engineering.md) (CONCEPT)
- [Dynamic Context Assembly](/entities/dynamic-context-assembly.md) (CONCEPT)
- [LiteLLM](/entities/litellm.md) (TOOL)
- [LongLLMLinguaPostprocessor](/entities/longllmlinguapostprocessor.md) (TOOL)
- [DocumentConverter()](/entities/documentconverter.md) (TOOL)
- [Surya](/entities/surya.md) (TOOL)
- [Mooncake](/entities/mooncake.md) (TOOL)
- [Prompt Compression](/entities/prompt-compression.md) (CONCEPT)
- [Document Structure-Based Chunking](/entities/document-structure-based-chunking.md) (CONCEPT)
- [LLMLingua-2](/entities/llmlingua-2.md) (TOOL)
- [Retrieval-Augmented Generation (RAG)](/entities/retrieval-augmented-generation-rag.md) (CONCEPT)
- [DéjàVu](/entities/d-j-vu.md) (TOOL)
- [EasyOCR](/entities/easyocr.md) (TOOL)
- [DoclingDocument](/entities/doclingdocument.md) (SYSTEM)
- [LlamaIndex Workflows](/entities/llamaindex-workflows.md) (TOOL)
- [Bills of Lading (BoLs)](/entities/bills-of-lading-bols.md) (CONCEPT)
- [HybridServe](/entities/hybridserve.md) (TOOL)
- [Semantic Chunking](/entities/semantic-chunking.md) (CONCEPT)
- [SemanticSplitterNodeParser](/entities/semanticsplitternodeparser.md) (TOOL)
- [Automatic Prefix Caching (APC)](/entities/automatic-prefix-caching-apc.md) (CONCEPT)
- [LangChain](/entities/langchain.md) (TOOL)
- [LlamaParse](/entities/llamaparse.md) (TOOL)
- [Mastra](/entities/mastra.md) (TOOL)
- [Tesseract](/entities/tesseract.md) (TOOL)
- [Quantization](/entities/quantization.md) (CONCEPT)
- [Visual Document Understanding (VDU)](/entities/visual-document-understanding-vdu.md) (CONCEPT)
- [Document Understanding Transformer (Donut)](/entities/document-understanding-transformer-donut.md) (TOOL)
- [LangGraph](/entities/langgraph.md) (TOOL)
- [DyCP (Dynamic Context Management)](/entities/dycp-dynamic-context-management.md) (CONCEPT)
- [LongLLMLingua](/entities/longllmlingua.md) (TOOL)
- [Model Context Protocols (MCP)](/entities/model-context-protocols-mcp.md) (CONCEPT)
- [vLLM](/entities/vllm.md) (TOOL)
- [Milvus](/entities/milvus.md) (TOOL)
- [semantic caching](/entities/semantic-caching.md) (CONCEPT)
- [Bill of Lading (BoL)](/entities/bill-of-lading-bol.md) (CONCEPT)
- [Hybrid Queries](/entities/hybrid-queries.md) (CONCEPT)
- [KV cache](/entities/kv-cache.md) (CONCEPT)
- [Reciprocal Rank Fusion (RRF)](/entities/reciprocal-rank-fusion-rrf.md) (CONCEPT)
- [Orchestration Frameworks](/entities/orchestration-frameworks.md) (CONCEPT)
- [Agent-to-Agent (A2A) Governance](/entities/agent-to-agent-a2a-governance.md) (CONCEPT)
- [Optical Character Recognition (OCR)](/entities/optical-character-recognition-ocr.md) (TOOL)
- [Redis](/entities/redis.md) (TOOL)

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
