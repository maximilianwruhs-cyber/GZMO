---
type: source
title: gzmo-chaos-engine-architecture-audit-and-behaviora
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# gzmo-chaos-engine-architecture-audit-and-behaviora

Ingested source summary (2026-06-08).

## Entities
- [[dreams|Dreams]] (CONCEPT)
- [[lorenz-attractor|Lorenz Attractor]] (CONCEPT)
- [[thought-cabinet|Thought Cabinet]] (CONCEPT)
- [[permanent-mutations|Permanent Mutations]] (CONCEPT)
- [[logistic-map|Logistic Map]] (CONCEPT)
- [[gzmo-chaos-engine|GZMO Chaos Engine]] (SYSTEM)
- [[livestream-md|LiveStream.md]] (CONCEPT)
- [[llmvalence|llmValence]] (CONCEPT)
- [[self-ask|Self-Ask]] (CONCEPT)
- [[chaosval|chaosVal]] (CONCEPT)
- [[engine-ts|engine.ts]] (TOOL)
- [[llmtemperature|llmTemperature]] (CONCEPT)
- [[pulse-ts|pulse.ts]] (TOOL)
- [[tension|tension]] (CONCEPT)
- [[cortisol|cortisol]] (CONCEPT)
- [[llmmaxtokens|llmMaxTokens]] (CONCEPT)
- [[index-ts|index.ts]] (TOOL)

## Relations
- GZMO Chaos Engine → USES → Lorenz Attractor
- GZMO Chaos Engine → USES → Logistic Map
- GZMO Chaos Engine → USES → Thought Cabinet
- GZMO Chaos Engine → USES → cortisol
- GZMO Chaos Engine → USES → tension
- Lorenz Attractor → RELATED_TO → llmTemperature
- Lorenz Attractor → RELATED_TO → llmMaxTokens
- Lorenz Attractor → RELATED_TO → llmValence
- Logistic Map → RELATED_TO → chaosVal
- Thought Cabinet → RELATED_TO → Permanent Mutations
- Permanent Mutations → RELATED_TO → Lorenz Attractor
- engine.ts → USES → llmTemperature
- engine.ts → USES → llmMaxTokens
- engine.ts → USES → llmValence
- index.ts → USES → Self-Ask
- index.ts → USES → GZMO Chaos Engine
- index.ts → USES → Dreams
- pulse.ts → USES → Self-Ask
- pulse.ts → USES → Thought Cabinet
- GZMO Chaos Engine → RELATED_TO → Self-Ask
- GZMO Chaos Engine → RELATED_TO → Dreams
- GZMO Chaos Engine → USES → LiveStream.md
