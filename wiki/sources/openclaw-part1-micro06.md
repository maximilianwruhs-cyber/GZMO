---
type: source
title: openclaw-part1-micro06
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# openclaw-part1-micro06

Ingested source summary (2026-06-09).

## Entities
- [FTS5](/entities/fts5.md) (TOOL)
- [Contriever](/entities/contriever.md) (TOOL)
- [OpenClaw](/entities/openclaw.md) (SYSTEM)
- [GZMO architecture](/entities/gzmo-architecture.md) (SYSTEM)
- [EU Artificial Intelligence Act (AI Act)](/entities/eu-artificial-intelligence-act-ai-act.md) (CONCEPT)
- [MEMORY.md](/entities/memory-md.md) (BOOK)
- [SOUL.md](/entities/soul-md.md) (BOOK)
- [Chief of Staff](/entities/chief-of-staff.md) (CONCEPT)
- [DREAMS.md](/entities/dreams-md.md) (BOOK)
- [sqlite-vec](/entities/sqlite-vec.md) (TOOL)
- [Dreaming-Engine](/entities/dreaming-engine.md) (SYSTEM)
- [GDPR (Datenschutz-Grundverordnung)](/entities/gdpr-datenschutz-grundverordnung.md) (CONCEPT)
- [LoRA (Low-Rank Adaptation)](/entities/lora-low-rank-adaptation.md) (TOOL)
- [Austrian Data Protection Act (Datenschutzgesetz)](/entities/austrian-data-protection-act-datenschutzgesetz.md) (CONCEPT)
- [Ebbinghaus's Forgetting Curve](/entities/ebbinghaus-s-forgetting-curve.md) (CONCEPT)
- [BM25](/entities/bm25.md) (CONCEPT)
- [HyDE (Hypothetical Document Embeddings)](/entities/hyde-hypothetical-document-embeddings.md) (TOOL)
- [Maximum Marginal Relevance (MMR)](/entities/maximum-marginal-relevance-mmr.md) (CONCEPT)
- [Cosine Similarity](/entities/cosine-similarity.md) (CONCEPT)
- [Strength Theory](/entities/strength-theory.md) (CONCEPT)
- [Privacy by Design](/entities/privacy-by-design.md) (CONCEPT)
- [Temporal Decay](/entities/temporal-decay.md) (CONCEPT)
- [Complementary Learning Systems (CLS) Theory](/entities/complementary-learning-systems-cls-theory.md) (CONCEPT)

## Relations
- OpenClaw → USES → sqlite-vec
- OpenClaw → USES → FTS5
- OpenClaw → USES → BM25
- OpenClaw → USES → Cosine Similarity
- OpenClaw → USES → Temporal Decay
- OpenClaw → PART_OF → Dreaming-Engine
- OpenClaw → USES → HyDE
- OpenClaw → USES → Contriever
- OpenClaw → USES → Maximum Marginal Relevance (MMR)
- OpenClaw → RELATED_TO → Ebbinghaus's Forgetting Curve
- OpenClaw → RELATED_TO → Strength Theory
- OpenClaw → USES → MEMORY.md
- OpenClaw → RELATED_TO → LoRA (Low-Rank Adaptation)
- OpenClaw → USES → SOUL.md
- OpenClaw → RELATED_TO → GDPR (Datenschutz-Grundverordnung)
- OpenClaw → RELATED_TO → Austrian Data Protection Act (Datenschutzgesetz)
- OpenClaw → RELATED_TO → EU Artificial Intelligence Act (AI Act)
- OpenClaw → RELATED_TO → Privacy by Design
- FTS5 → USES → BM25
- GZMO architecture → USES → Temporal Decay
- Dreaming-Engine → USES → DREAMS.md
- Dreaming-Engine → USES → HyDE
- Dreaming-Engine → USES → Contriever
- Dreaming-Engine → USES → Maximum Marginal Relevance (MMR)
- Dreaming-Engine → RELATED_TO → Ebbinghaus's Forgetting Curve
- Dreaming-Engine → RELATED_TO → Strength Theory
- Complementary Learning Systems (CLS) Theory → RELATED_TO → OpenClaw
- MEMORY.md → RELATED_TO → Temporal Decay
