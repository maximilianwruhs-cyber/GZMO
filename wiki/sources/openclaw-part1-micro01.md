---
type: source
title: openclaw-part1-micro01
created: 2026-06-10
updated: 2026-06-10
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# openclaw-part1-micro01

Ingested source summary (2026-06-10).

## Entities
- [DREAMS.md](/entities/dreams-md.md) (SYSTEM)
- [Heartbeat](/entities/heartbeat.md) (SYSTEM)
- [Tages-Logs](/entities/tages-logs.md) (SYSTEM)
- [Atkinson-Shiffrin-Modell](/entities/atkinson-shiffrin-modell.md) (CONCEPT)
- [MEMORY.md](/entities/memory-md.md) (SYSTEM)
- [OpenClaw 4.5](/entities/openclaw-4-5.md) (SYSTEM)
- [FTS5-Erweiterung](/entities/fts5-erweiterung.md) (TOOL)
- [Temporal Decay](/entities/temporal-decay.md) (CONCEPT)
- [Dreaming-Prozess](/entities/dreaming-prozess.md) (SYSTEM)
- [HyDE](/entities/hyde.md) (TOOL)
- [Metakognition](/entities/metakognition.md) (CONCEPT)
- [Context Overload](/entities/context-overload.md) (CONCEPT)
- [MMR-Algorithmus](/entities/mmr-algorithmus.md) (TOOL)
- [Hybride Suche](/entities/hybride-suche.md) (SYSTEM)
- [sqlite-vec](/entities/sqlite-vec.md) (TOOL)
- [Silent Turns](/entities/silent-turns.md) (CONCEPT)

## Relations
- OpenClaw 4.5 → USES → Heartbeat
- OpenClaw 4.5 → USES → Tages-Logs
- OpenClaw 4.5 → USES → MEMORY.md
- OpenClaw 4.5 → USES → Dreaming-Prozess
- Hybride Suche → USES → sqlite-vec
- Hybride Suche → USES → FTS5-Erweiterung
- Dreaming-Prozess → USES → HyDE
- Dreaming-Prozess → USES → MMR-Algorithmus
- Dreaming-Prozess → RELATED_TO → DREAMS.md
