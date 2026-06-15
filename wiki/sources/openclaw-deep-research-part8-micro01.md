---
type: source
title: openclaw-deep-research-part8-micro01
created: 2026-06-10
updated: 2026-06-10
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# openclaw-deep-research-part8-micro01

Ingested source summary (2026-06-10).

## Entities
- [[capital-one|Capital One]] (ORGANIZATION)
- [[autogen|AutoGen]] (SYSTEM)
- [[pinecone|Pinecone]] (TOOL)
- [[microsoft|Microsoft]] (ORGANIZATION)
- [[langsmith|LangSmith]] (TOOL)
- [[deloitte|Deloitte]] (ORGANIZATION)
- [[redis|Redis]] (TOOL)
- [[azure|Azure]] (SYSTEM)
- [[kubernetes|Kubernetes]] (SYSTEM)
- [[milvus|Milvus]] (TOOL)
- [[faiss|FAISS]] (TOOL)
- [[crewai|CrewAI]] (SYSTEM)
- [[langgraph|LangGraph]] (SYSTEM)
- [[ibm|IBM]] (ORGANIZATION)
- [[shopify|Shopify]] (ORGANIZATION)
- [[aws-lambda|AWS Lambda]] (SYSTEM)
- [[langchain|LangChain]] (SYSTEM)
- [[postgresql|PostgreSQL]] (TOOL)
- [[sqlite|SQLite]] (TOOL)

## Relations
- LangGraph → PART_OF → LangChain
- LangChain → USES → Pinecone
- LangChain → USES → FAISS
- LangChain → USES → Milvus
- LangChain → USES → Redis
- LangChain → USES → SQLite
- LangChain → USES → PostgreSQL
- LangChain → USES → LangSmith
- AutoGen → AUTHORED_BY → Microsoft
- AutoGen → USES → Azure
- AutoGen → USES → SQLite
- AutoGen → USES → Redis
- LangChain → RELATED_TO → IBM
- LangChain → RELATED_TO → Capital One
- CrewAI → RELATED_TO → Shopify
