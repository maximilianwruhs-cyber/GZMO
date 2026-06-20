---
type: entity
title: SQLite
created: 2026-06-08
updated: 2026-06-10
sources: 17
tags: []
status: draft
gzmo_synthetic: true
---

















# SQLite

Type: SYSTEM

## From [allgemeine-informationen](/entities/allgemeine-informationen.md) (2026-06-08)
- It is a backend option for session persistence.

## From [ai-research-part7](/entities/ai-research-part7.md) (2026-06-08)
- Claw Empire operates entirely locally via SQLite databases.
- It ensures maximum data privacy and eliminates cloud dependencies.

## From [architectural-framework-and-development-of-pi-codi](/entities/architectural-framework-and-development-of-pi-codi.md) (2026-06-08)
- Local SQLite databases are used by advanced extensions for persistent memory.

## From [architectural-strategy-for-stealthy-portable-cli-a](/entities/architectural-strategy-for-stealthy-portable-cli-a.md) (2026-06-08)
- SQLCipher provides robust page-level encryption for SQLite databases using 256-bit AES.
- Avoid SQLite for USBs: SQLite utilizes Write-Ahead Logging (.wal) and shared memory (.shm) files.
- If a user abruptly unplugs the USB drive while the agent is running, these lock files become orphaned, resulting in database corruption.

## From [cybernetics-and-mythos-the-architecture-of-intell-part1](/entities/cybernetics-and-mythos-the-architecture-of-intell-part1.md) (2026-06-08)
- It is a type of hybrid memory database.
- It is used to continually refresh immediate context.
- It is part of the Advanced Dialogic Specification Protocol for context maintenance.

## From [openclaw-deep-research-part6](/entities/openclaw-deep-research-part6.md) (2026-06-08)
- Used for OpenClaw's long-term memory management.
- Its FTS5 extension is used for keyword matching in hybrid search.

## From [openclaw-deep-research-part12](/entities/openclaw-deep-research-part12.md) (2026-06-08)
- Used for Vector Search and Keyword Match in memory retrieval.
- Employs the FTS5 extension for keyword matching.

## From [drive-research-du-hast-gesagt-part1](/entities/drive-research-du-hast-gesagt-part1.md) (2026-06-08)
- Used to store vector databases locally.

## From [drive-research-hermes-session-storage-migration-analysis](/entities/drive-research-hermes-session-storage-migration-analysis.md) (2026-06-08)
- Used for metadata, token counts, billing information, and full-text searches within the Hermes framework.
- Located at ~/.hermes/state.db.
- Supports FTS5 for full-text searches.
- Has a schema version tracked by SCHEMA_VERSION constant.
- Central tables include sessions, messages, messages_fts, messages_fts_trigram, and state_meta.
- Can be used to store thematic segments in a separate table (session_segments).
- Initialized in Write-Ahead Logging mode (PRAGMA journal_mode=WAL) for concurrency.
- Can fall back to journal_mode=DELETE on network file systems incompatible with POSIX locking protocols.
- Used by Hermes for metadata, routing telemetry, and session lineage.
- Used by Gobii as a proactive, first-class tool for agents.
- Each agent in Gobii gets its own embedded SQL database.
- Hermes's infrastructure is already utilized.
- Core of Hermes's system architecture.
- Used for session transcript accumulation.

## From [drive-research-hermes-system-untersuchung-und-erweiterung](/entities/drive-research-hermes-system-untersuchung-und-erweiterung.md) (2026-06-08)
- Used in the Three-Tier Memory Architecture.
- Stores session segments in the session_segments table.
- Supports Full-Text Search (FTS5) for segment searching.
- Used for the session_segments table.
- Operates on the internal system context.
- Secures SQLite databases.

## From [drive-research-pi-coding-agent-ecosystem-tier-list](/entities/drive-research-pi-coding-agent-ecosystem-tier-list.md) (2026-06-08)
- Used by pi-local-rag to store embeddings and full text.
- Supports FTS5 for efficient querying.
- Stores data for the RAG ingestion pipeline.
- Stores Xenova/all-MiniLM-L6-v2 embeddings for pi-local-rag.

## From [gzmo-soul-merged-new-part1](/entities/gzmo-soul-merged-new-part1.md) (2026-06-09)
- Database that Python task managers might use for data persistence

## From [building-a-private-local-ai-development-environmen-micro01](/entities/building-a-private-local-ai-development-environmen-micro01.md) (2026-06-09)
- Can be accessed by MCP servers for schema insight

## From [drive-research-redefining-agentic-soulmd-to-dialog-micro04](/entities/drive-research-redefining-agentic-soulmd-to-dialog-micro04.md) (2026-06-09)
- Used in hybrid memory databases.
- Part of the Context Layer implementation.

## From [openclaw-deep-research-part1-micro06](/entities/openclaw-deep-research-part1-micro06.md) (2026-06-10)
- The database engine used for the persistence layer via the sqlite-vec extension.

## From [openclaw-deep-research-part8-micro01](/entities/openclaw-deep-research-part8-micro01.md) (2026-06-10)
- Used for session management and persistent conversation states

## From [openclaw-deep-research-part9-micro01](/entities/openclaw-deep-research-part9-micro01.md) (2026-06-10)
- Used for semantic vector search storage.
