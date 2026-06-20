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
- [tokio-cron-scheduler](/entities/tokio-cron-scheduler.md) (TOOL)
- [openclaw-daemon](/entities/openclaw-daemon.md) (PROJECT)
- [openclaw-skills](/entities/openclaw-skills.md) (PROJECT)
- [Cargo Workspace](/entities/cargo-workspace.md) (TOOL)
- [Metacognitive Guard](/entities/metacognitive-guard.md) (CONCEPT)
- [openclaw-identity](/entities/openclaw-identity.md) (PROJECT)
- [memory/YYYY-MM-DD.md](/entities/memory-yyyy-mm-dd-md.md) (CONCEPT)
- [tsvector](/entities/tsvector.md) (CONCEPT)
- [LLM](/entities/llm.md) (SYSTEM)
- [serde_yaml](/entities/serde-yaml.md) (TOOL)
- [SOUL.md](/entities/soul-md.md) (CONCEPT)
- [Maximal Marginal Relevance (MMR)](/entities/maximal-marginal-relevance-mmr.md) (CONCEPT)
- [openclaw-cli](/entities/openclaw-cli.md) (PROJECT)
- [bollard](/entities/bollard.md) (TOOL)
- [ort](/entities/ort.md) (TOOL)
- [openclaw-memory](/entities/openclaw-memory.md) (PROJECT)
- [LlmClient](/entities/llmclient.md) (SYSTEM)
- [openclaw-dreams](/entities/openclaw-dreams.md) (PROJECT)
- [Rust](/entities/rust.md) (TOOL)
- [SKILL.md](/entities/skill-md.md) (CONCEPT)
- [pulldown-cmark](/entities/pulldown-cmark.md) (TOOL)
- [fastembed](/entities/fastembed.md) (TOOL)
- [notify](/entities/notify.md) (TOOL)
- [openclaw-core](/entities/openclaw-core.md) (PROJECT)
- [Docker](/entities/docker.md) (SYSTEM)
- [PostgreSQL](/entities/postgresql.md) (SYSTEM)
- [DREAMS.md](/entities/dreams-md.md) (CONCEPT)
- [Tabula Rasa](/entities/tabula-rasa.md) (CONCEPT)
- [pgvector](/entities/pgvector.md) (TOOL)
- [ndarray](/entities/ndarray.md) (TOOL)
- [autoDream](/entities/autodream.md) (CONCEPT)
- [the-cognitive-architecture-of-openclaw-agents.md](/entities/the-cognitive-architecture-of-openclaw-agents-md.md) (BOOK)
- [CheapCheck trait](/entities/cheapcheck-trait.md) (CONCEPT)
- [sqlx](/entities/sqlx.md) (TOOL)
- [HEARTBEAT.md](/entities/heartbeat-md.md) (CONCEPT)
- [Atkinson-Shiffrin decay formula](/entities/atkinson-shiffrin-decay-formula.md) (CONCEPT)

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
