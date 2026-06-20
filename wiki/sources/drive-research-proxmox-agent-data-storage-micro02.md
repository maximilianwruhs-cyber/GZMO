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
- [EdgeQuake](/entities/edgequake.md) (TOOL)
- [neo4j-graphrag-python](/entities/neo4j-graphrag-python.md) (TOOL)
- [GLiNER](/entities/gliner.md) (TOOL)
- [samanhappy/mcphub](/entities/samanhappy-mcphub.md) (TOOL)
- [EHRAG](/entities/ehrag.md) (TOOL)
- [ravitemer/mcp-hub](/entities/ravitemer-mcp-hub.md) (TOOL)
- [QdrantClient](/entities/qdrantclient.md) (TOOL)
- [awesome-mcp-gateways](/entities/awesome-mcp-gateways.md) (TOOL)
- [Neo4j Graph DB](/entities/neo4j-graph-db.md) (TOOL)
- [Git Engine](/entities/git-engine.md) (TOOL)
- [Express API](/entities/express-api.md) (TOOL)
- [POSIX Filesystem](/entities/posix-filesystem.md) (TOOL)
- [LightRAG Framework](/entities/lightrag-framework.md) (TOOL)
- [Agent Memory Server](/entities/agent-memory-server.md) (TOOL)
- [Qdrant Vector DB](/entities/qdrant-vector-db.md) (TOOL)
- [Redis KV Cache](/entities/redis-kv-cache.md) (TOOL)
- [Network File System (NFS)](/entities/network-file-system-nfs.md) (TOOL)
- [sentence-transformers/all-MiniLM-L6-v2](/entities/sentence-transformers-all-minilm-l6-v2.md) (TOOL)
- [AIRIS MCP Gateway](/entities/airis-mcp-gateway.md) (TOOL)
- [LXC 100](/entities/lxc-100.md) (SYSTEM)
- [Model Context Protocol (MCP)](/entities/model-context-protocol-mcp.md) (CONCEPT)
- [SpaCy](/entities/spacy.md) (TOOL)
- [LXC 102](/entities/lxc-102.md) (SYSTEM)
- [FastMCP](/entities/fastmcp.md) (TOOL)
- [mcphub CLI](/entities/mcphub-cli.md) (TOOL)
- [Node.js Runtime](/entities/node-js-runtime.md) (TOOL)
- [LazyGraphRAG](/entities/lazygraphrag.md) (TOOL)
- [Proxmox](/entities/proxmox.md) (SYSTEM)
- [LXC 101](/entities/lxc-101.md) (SYSTEM)
- [Filesystem MCP Server](/entities/filesystem-mcp-server.md) (TOOL)
- [Git MCP Server](/entities/git-mcp-server.md) (TOOL)

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
