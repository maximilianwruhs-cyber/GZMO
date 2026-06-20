---
type: source
title: obolus-micro01
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# obolus-micro01

Ingested source summary (2026-06-09).

## Entities
- [Validator (Richter)](/entities/validator-richter.md) (SYSTEM)
- [Obolus-Watt-Ökonomie](/entities/obolus-watt-konomie.md) (CONCEPT)
- [Miner (Agenten)](/entities/miner-agenten.md) (SYSTEM)
- [Obulus](/entities/obulus.md) (SYSTEM)
- [Proof-of-Efficiency](/entities/proof-of-efficiency.md) (CONCEPT)
- [Fitness-Scorer](/entities/fitness-scorer.md) (TOOL)
- [Genom](/entities/genom.md) (CONCEPT)
- [Forge (Evolutions-Engine)](/entities/forge-evolutions-engine.md) (SYSTEM)

## Relations
- Obolus-Watt-Ökonomie → PART_OF → Obulus
- Obulus → PART_OF → Miner (Agenten)
- Obulus → PART_OF → Validator (Richter)
- Obulus → PART_OF → Forge (Evolutions-Engine)
- Obulus → RELATED_TO → Obolus-Watt-Ökonomie
- Obulus → RELATED_TO → Proof-of-Efficiency
- Forge (Evolutions-Engine) → RELATED_TO → Genom
- Obulus → USES → Fitness-Scorer
