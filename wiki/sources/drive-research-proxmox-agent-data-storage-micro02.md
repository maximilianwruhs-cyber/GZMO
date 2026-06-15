---
type: source
title: drive-research-proxmox-agent-data-storage-micro02
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# drive-research-proxmox-agent-data-storage-micro02

Ingested source summary (2026-06-09).

## Entities
- [[edgequake|EdgeQuake]] (TOOL)
- [[neo4j-graphrag-python|neo4j-graphrag-python]] (TOOL)
- [[gliner|GLiNER]] (TOOL)
- [[samanhappy-mcphub|samanhappy/mcphub]] (TOOL)
- [[ehrag|EHRAG]] (TOOL)
- [[ravitemer-mcp-hub|ravitemer/mcp-hub]] (TOOL)
- [[qdrantclient|QdrantClient]] (TOOL)
- [[awesome-mcp-gateways|awesome-mcp-gateways]] (TOOL)
- [[neo4j-graph-db|Neo4j Graph DB]] (TOOL)
- [[git-engine|Git Engine]] (TOOL)
- [[express-api|Express API]] (TOOL)
- [[posix-filesystem|POSIX Filesystem]] (TOOL)
- [[lightrag-framework|LightRAG Framework]] (TOOL)
- [[agent-memory-server|Agent Memory Server]] (TOOL)
- [[qdrant-vector-db|Qdrant Vector DB]] (TOOL)
- [[redis-kv-cache|Redis KV Cache]] (TOOL)
- [[network-file-system-nfs|Network File System (NFS)]] (TOOL)
- [[sentence-transformers-all-minilm-l6-v2|sentence-transformers/all-MiniLM-L6-v2]] (TOOL)
- [[airis-mcp-gateway|AIRIS MCP Gateway]] (TOOL)
- [[lxc-100|LXC 100]] (SYSTEM)
- [[model-context-protocol-mcp|Model Context Protocol (MCP)]] (CONCEPT)
- [[spacy|SpaCy]] (TOOL)
- [[lxc-102|LXC 102]] (SYSTEM)
- [[fastmcp|FastMCP]] (TOOL)
- [[mcphub-cli|mcphub CLI]] (TOOL)
- [[node-js-runtime|Node.js Runtime]] (TOOL)
- [[lazygraphrag|LazyGraphRAG]] (TOOL)
- [[proxmox|Proxmox]] (SYSTEM)
- [[lxc-101|LXC 101]] (SYSTEM)
- [[filesystem-mcp-server|Filesystem MCP Server]] (TOOL)
- [[git-mcp-server|Git MCP Server]] (TOOL)

## Relations
- LXC 100 → PART_OF → Proxmox
- LXC 101 → PART_OF → Proxmox
- LXC 102 → PART_OF → Proxmox
- LXC 100 → USES → POSIX Filesystem
- LXC 100 → USES → Git Engine
- LXC 100 → USES → FastMCP
- LXC 101 → USES → Neo4j Graph DB
- LXC 101 → USES → Qdrant Vector DB
- LXC 101 → USES → Redis KV Cache
- LXC 102 → USES → Node.js Runtime
- LXC 102 → USES → Express API
- LXC 102 → USES → ravitemer/mcp-hub
- LXC 102 → USES → mcphub CLI
- LXC 100 → USES → Filesystem MCP Server
- LXC 100 → USES → Git MCP Server
- LXC 101 → USES → neo4j-graphrag-python
- LXC 101 → USES → LightRAG Framework
- LXC 101 → USES → EdgeQuake
- LXC 101 → USES → Agent Memory Server
- LXC 102 → USES → samanhappy/mcphub
- LXC 102 → USES → awesome-mcp-gateways
- LXC 102 → USES → AIRIS MCP Gateway
- Model Context Protocol (MCP) → RELATED_TO → LXC 100
- Model Context Protocol (MCP) → RELATED_TO → LXC 101
- Model Context Protocol (MCP) → RELATED_TO → LXC 102
- neo4j-graphrag-python → RELATED_TO → LXC 101
- LXC 100 → USES → Network File System (NFS)
- LXC 101 → USES → GLiNER
- LXC 101 → USES → SpaCy
- LXC 101 → USES → sentence-transformers/all-MiniLM-L6-v2
- LXC 101 → USES → EHRAG
- LXC 101 → USES → LazyGraphRAG
- neo4j-graphrag-python → USES → QdrantClient
