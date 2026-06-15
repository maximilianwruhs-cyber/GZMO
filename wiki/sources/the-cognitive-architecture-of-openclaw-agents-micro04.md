---
type: source
title: the-cognitive-architecture-of-openclaw-agents-micro04
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# the-cognitive-architecture-of-openclaw-agents-micro04

Ingested source summary (2026-06-09).

## Entities
- [[tokio-cron-scheduler|tokio-cron-scheduler]] (TOOL)
- [[openclaw-daemon|openclaw-daemon]] (PROJECT)
- [[openclaw-skills|openclaw-skills]] (PROJECT)
- [[cargo-workspace|Cargo Workspace]] (TOOL)
- [[metacognitive-guard|Metacognitive Guard]] (CONCEPT)
- [[openclaw-identity|openclaw-identity]] (PROJECT)
- [[memory-yyyy-mm-dd-md|memory/YYYY-MM-DD.md]] (CONCEPT)
- [[tsvector|tsvector]] (CONCEPT)
- [[llm|LLM]] (SYSTEM)
- [[serde-yaml|serde_yaml]] (TOOL)
- [[soul-md|SOUL.md]] (CONCEPT)
- [[maximal-marginal-relevance-mmr|Maximal Marginal Relevance (MMR)]] (CONCEPT)
- [[openclaw-cli|openclaw-cli]] (PROJECT)
- [[bollard|bollard]] (TOOL)
- [[ort|ort]] (TOOL)
- [[openclaw-memory|openclaw-memory]] (PROJECT)
- [[llmclient|LlmClient]] (SYSTEM)
- [[openclaw-dreams|openclaw-dreams]] (PROJECT)
- [[rust|Rust]] (TOOL)
- [[skill-md|SKILL.md]] (CONCEPT)
- [[pulldown-cmark|pulldown-cmark]] (TOOL)
- [[fastembed|fastembed]] (TOOL)
- [[notify|notify]] (TOOL)
- [[openclaw-core|openclaw-core]] (PROJECT)
- [[docker|Docker]] (SYSTEM)
- [[postgresql|PostgreSQL]] (SYSTEM)
- [[dreams-md|DREAMS.md]] (CONCEPT)
- [[tabula-rasa|Tabula Rasa]] (CONCEPT)
- [[pgvector|pgvector]] (TOOL)
- [[ndarray|ndarray]] (TOOL)
- [[autodream|autoDream]] (CONCEPT)
- [[the-cognitive-architecture-of-openclaw-agents-md|the-cognitive-architecture-of-openclaw-agents.md]] (BOOK)
- [[cheapcheck-trait|CheapCheck trait]] (CONCEPT)
- [[sqlx|sqlx]] (TOOL)
- [[heartbeat-md|HEARTBEAT.md]] (CONCEPT)
- [[atkinson-shiffrin-decay-formula|Atkinson-Shiffrin decay formula]] (CONCEPT)

## Relations
- the-cognitive-architecture-of-openclaw-agents.md → USES → Rust
- Rust → USES → tokio-cron-scheduler
- Cargo Workspace → PART_OF → openclaw-core
- Cargo Workspace → PART_OF → openclaw-identity
- Cargo Workspace → PART_OF → openclaw-daemon
- Cargo Workspace → PART_OF → openclaw-skills
- Cargo Workspace → PART_OF → openclaw-memory
- Cargo Workspace → PART_OF → openclaw-dreams
- Cargo Workspace → PART_OF → openclaw-cli
- openclaw-identity → USES → SOUL.md
- openclaw-identity → USES → Tabula Rasa
- openclaw-identity → USES → notify
- openclaw-daemon → USES → HEARTBEAT.md
- openclaw-daemon → USES → tokio-cron-scheduler
- openclaw-daemon → USES → CheapCheck trait
- openclaw-daemon → USES → LlmClient
- openclaw-skills → USES → SKILL.md
- openclaw-skills → USES → bollard
- openclaw-skills → USES → Docker
- openclaw-skills → USES → Metacognitive Guard
- openclaw-memory → USES → memory/YYYY-MM-DD.md
- openclaw-memory → USES → PostgreSQL
- openclaw-memory → USES → tokio-cron-scheduler
- PostgreSQL → USES → pgvector
- PostgreSQL → USES → tsvector
- openclaw-dreams → USES → autoDream
- openclaw-dreams → USES → DREAMS.md
- openclaw-cli → USES → openclaw-memory
- Rust → USES → Cargo Workspace
- Rust → USES → pulldown-cmark
- Rust → USES → serde_yaml
- Rust → USES → notify
- Rust → USES → ndarray
- Rust → USES → fastembed
- Rust → USES → ort
- Rust → USES → bollard
- Rust → USES → sqlx
- autoDream → USES → fastembed
- autoDream → USES → ndarray
- autoDream → USES → Maximal Marginal Relevance (MMR)
- PostgreSQL → USES → sqlx
- sqlx → USES → Atkinson-Shiffrin decay formula
- the-cognitive-architecture-of-openclaw-agents.md → USES → LLM
