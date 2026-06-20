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
- [Dreams](/entities/dreams.md) (CONCEPT)
- [Lorenz Attractor](/entities/lorenz-attractor.md) (CONCEPT)
- [Thought Cabinet](/entities/thought-cabinet.md) (CONCEPT)
- [Permanent Mutations](/entities/permanent-mutations.md) (CONCEPT)
- [Logistic Map](/entities/logistic-map.md) (CONCEPT)
- [GZMO Chaos Engine](/entities/gzmo-chaos-engine.md) (SYSTEM)
- [LiveStream.md](/entities/livestream-md.md) (CONCEPT)
- [llmValence](/entities/llmvalence.md) (CONCEPT)
- [Self-Ask](/entities/self-ask.md) (CONCEPT)
- [chaosVal](/entities/chaosval.md) (CONCEPT)
- [engine.ts](/entities/engine-ts.md) (TOOL)
- [llmTemperature](/entities/llmtemperature.md) (CONCEPT)
- [pulse.ts](/entities/pulse-ts.md) (TOOL)
- [tension](/entities/tension.md) (CONCEPT)
- [cortisol](/entities/cortisol.md) (CONCEPT)
- [llmMaxTokens](/entities/llmmaxtokens.md) (CONCEPT)
- [index.ts](/entities/index-ts.md) (TOOL)

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
