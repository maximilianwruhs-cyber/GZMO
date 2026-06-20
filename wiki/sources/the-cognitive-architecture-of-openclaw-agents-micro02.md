---
type: source
title: the-cognitive-architecture-of-openclaw-agents-micro02
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# the-cognitive-architecture-of-openclaw-agents-micro02

Ingested source summary (2026-06-09).

## Entities
- [GZMO persona](/entities/gzmo-persona.md) (CONCEPT)
- [Lazy Loading architecture](/entities/lazy-loading-architecture.md) (CONCEPT)
- [SKILL.md](/entities/skill-md.md) (SYSTEM)
- [HEARTBEAT.md](/entities/heartbeat-md.md) (SYSTEM)
- [large language models](/entities/large-language-models.md) (SYSTEM)
- [Moltbook](/entities/moltbook.md) (SYSTEM)
- [Hypothetical Document Embeddings (HyDE)](/entities/hypothetical-document-embeddings-hyde.md) (TOOL)
- [autoDream cycle](/entities/autodream-cycle.md) (CONCEPT)
- [nemawashi](/entities/nemawashi.md) (CONCEPT)
- [memory/YYYY-MM-DD.md](/entities/memory-yyyy-mm-dd-md.md) (SYSTEM)
- [OpenClaw](/entities/openclaw.md) (PROJECT)
- [Tabula Rasa](/entities/tabula-rasa.md) (CONCEPT)
- [Gardening directive](/entities/gardening-directive.md) (CONCEPT)
- [SOUL.md](/entities/soul-md.md) (SYSTEM)
- [DREAMS.md](/entities/dreams-md.md) (SYSTEM)
- [NO_REPLY protocol](/entities/no-reply-protocol.md) (CONCEPT)

## Relations
- SOUL.md → DEFINES → GZMO persona
- SOUL.md → RELATED_TO → Tabula Rasa
- SOUL.md → RELATED_TO → Gardening directive
- Gardening directive → RELATED_TO → nemawashi
- HEARTBEAT.md → USES → large language models
- SKILL.md → RELATED_TO → Lazy Loading architecture
- memory/YYYY-MM-DD.md → RELATED_TO → NO_REPLY protocol
- DREAMS.md → PART_OF → autoDream cycle
- autoDream cycle → USES → large language models
- autoDream cycle → USES → Hypothetical Document Embeddings (HyDE)
- Hypothetical Document Embeddings (HyDE) → USES → memory/YYYY-MM-DD.md
- OpenClaw → RELATED_TO → Moltbook
