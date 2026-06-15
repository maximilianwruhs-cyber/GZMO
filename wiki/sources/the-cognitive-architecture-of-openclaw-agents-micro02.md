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
- [[gzmo-persona|GZMO persona]] (CONCEPT)
- [[lazy-loading-architecture|Lazy Loading architecture]] (CONCEPT)
- [[skill-md|SKILL.md]] (SYSTEM)
- [[heartbeat-md|HEARTBEAT.md]] (SYSTEM)
- [[large-language-models|large language models]] (SYSTEM)
- [[moltbook|Moltbook]] (SYSTEM)
- [[hypothetical-document-embeddings-hyde|Hypothetical Document Embeddings (HyDE)]] (TOOL)
- [[autodream-cycle|autoDream cycle]] (CONCEPT)
- [[nemawashi|nemawashi]] (CONCEPT)
- [[memory-yyyy-mm-dd-md|memory/YYYY-MM-DD.md]] (SYSTEM)
- [[openclaw|OpenClaw]] (PROJECT)
- [[tabula-rasa|Tabula Rasa]] (CONCEPT)
- [[gardening-directive|Gardening directive]] (CONCEPT)
- [[soul-md|SOUL.md]] (SYSTEM)
- [[dreams-md|DREAMS.md]] (SYSTEM)
- [[no-reply-protocol|NO_REPLY protocol]] (CONCEPT)

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
