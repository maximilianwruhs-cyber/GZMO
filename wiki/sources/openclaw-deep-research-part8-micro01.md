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
- [Capital One](/entities/capital-one.md) (ORGANIZATION)
- [AutoGen](/entities/autogen.md) (SYSTEM)
- [Pinecone](/entities/pinecone.md) (TOOL)
- [Microsoft](/entities/microsoft.md) (ORGANIZATION)
- [LangSmith](/entities/langsmith.md) (TOOL)
- [Deloitte](/entities/deloitte.md) (ORGANIZATION)
- [Redis](/entities/redis.md) (TOOL)
- [Azure](/entities/azure.md) (SYSTEM)
- [Kubernetes](/entities/kubernetes.md) (SYSTEM)
- [Milvus](/entities/milvus.md) (TOOL)
- [FAISS](/entities/faiss.md) (TOOL)
- [CrewAI](/entities/crewai.md) (SYSTEM)
- [LangGraph](/entities/langgraph.md) (SYSTEM)
- [IBM](/entities/ibm.md) (ORGANIZATION)
- [Shopify](/entities/shopify.md) (ORGANIZATION)
- [AWS Lambda](/entities/aws-lambda.md) (SYSTEM)
- [LangChain](/entities/langchain.md) (SYSTEM)
- [PostgreSQL](/entities/postgresql.md) (TOOL)
- [SQLite](/entities/sqlite.md) (TOOL)

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
