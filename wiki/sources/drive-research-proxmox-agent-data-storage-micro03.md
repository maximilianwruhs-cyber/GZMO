---
type: source
title: drive-research-proxmox-agent-data-storage-micro03
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# drive-research-proxmox-agent-data-storage-micro03

Ingested source summary (2026-06-09).

## Entities
- [[semantic-graphrag-implementation-guide|Semantic GraphRAG Implementation Guide]] (CONCEPT)
- [[lxc-100|LXC 100]] (SYSTEM)
- [[neo4j-in-coding-tools-editors|Neo4j in Coding Tools & Editors]] (BOOK)
- [[qdrant-model-context-protocol-mcp-server-implementation|Qdrant Model Context Protocol (MCP) server implementation]] (TOOL)
- [[lxc-102-hub|LXC 102 (Hub)]] (SYSTEM)
- [[neo4j-get-schema|neo4j__get-schema]] (TOOL)
- [[ai-client|AI Client]] (SYSTEM)
- [[1999azzar-filesystem-mcp-server|1999AZZAR/filesystem-mcp-server]] (SYSTEM)
- [[json-rpc|JSON-RPC]] (CONCEPT)
- [[lxc-101|LXC 101]] (SYSTEM)
- [[filesystem-write-file|filesystem__write_file]] (TOOL)
- [[git-mcp-server|Git MCP server]] (SYSTEM)
- [[cypher|Cypher]] (CONCEPT)
- [[fastembed|FastEmbed]] (TOOL)
- [[git-create-commit|git__create_commit]] (TOOL)
- [[orm-class-model|ORM class model]] (CONCEPT)
- [[bearer-token|Bearer token]] (CONCEPT)

## Relations
- AI Client → USES → LXC 102 (Hub)
- AI Client → USES → ORM class model
- LXC 102 (Hub) → USES → Bearer token
- LXC 102 (Hub) → USES → neo4j__get-schema
- Neo4j in Coding Tools & Editors → USES → Cypher
- LXC 101 → USES → Neo4j in Coding Tools & Editors
- LXC 100 → PART_OF → 1999AZZAR/filesystem-mcp-server
- LXC 100 → PART_OF → Git MCP server
- 1999AZZAR/filesystem-mcp-server → PART_OF → LXC 100
- Git MCP server → PART_OF → LXC 100
- AI Client → USES → filesystem__write_file
- AI Client → USES → git__create_commit
- Qdrant Model Context Protocol (MCP) server implementation → RELATED_TO → Semantic GraphRAG Implementation Guide
