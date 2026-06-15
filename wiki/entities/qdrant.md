---
type: entity
title: Qdrant
created: 2026-06-08
updated: 2026-06-10
sources: 10
tags: []
status: draft
gzmo_synthetic: true
---










# Qdrant

Type: SYSTEM

## From [[openclaw-part2|openclaw-part2]] (2026-06-08)
- An external vector database for Retrieval-Augmented Generation (RAG).
- OpenClaw explicitly eschews dependency on such databases by default.

## From [[aether-grid-micro01|aether-grid-micro01]] (2026-06-09)
- Distributed Vector DB.
- Deployed in Distributed Mode via Helm.
- Local Edge instances use Memory-Mapped-Files (mmap) on NVMe.
- Indexing spikes occur after pairing.

## From [[aether-grid-micro04|aether-grid-micro04]] (2026-06-09)
- Used as a Vector-DB.
- Supports distributed mode.
- Hot vectors are cached in Edge-Node RAM, long-term memory on GH200.

## From [[aether-grid-micro03|aether-grid-micro03]] (2026-06-09)
- Database to be deployed in Distributed Mode on GH200 in Phase 2.

## From [[drive-research-linux-gaming-and-ai-build-guide-micro02|drive-research-linux-gaming-and-ai-build-guide-micro02]] (2026-06-09)
- A local vector database.
- Can consume 20-30GB of RAM in an idle state.
- Part of the AI orchestration layer.

## From [[drive-research-linux-gaming-and-ai-build-guide-micro05|drive-research-linux-gaming-and-ai-build-guide-micro05]] (2026-06-09)
- Local vector database.
- Consumes significant RAM in an AI orchestration layer.

## From [[prompt-agent-engineering-part4-micro01|prompt-agent-engineering-part4-micro01]] (2026-06-09)
- A vector database.
- Partitioned into strictly isolated customer namespaces.
- Used for Knowledge Injection in the Core Cluster.

## From [[the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro02|the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro02]] (2026-06-09)
- A vector database that custom API connectors can interface with.

## From [[prompt-agent-engineering-part4-micro02|prompt-agent-engineering-part4-micro02]] (2026-06-10)
- Used for indexing strategy
- Can be instantiated in a Docker container
- Supports TLS encryption and namespace isolation

## From [[prompt-agent-engineering-part4-micro03|prompt-agent-engineering-part4-micro03]] (2026-06-10)
- Vector database used for RAG-Deep-Core
- Runs in High-Performance Mode with TLS encryption
- Uses isolated namespaces for multi-tenancy
