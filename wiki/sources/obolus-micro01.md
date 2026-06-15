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
- [[validator-richter|Validator (Richter)]] (SYSTEM)
- [[obolus-watt-konomie|Obolus-Watt-Ökonomie]] (CONCEPT)
- [[miner-agenten|Miner (Agenten)]] (SYSTEM)
- [[obulus|Obulus]] (SYSTEM)
- [[proof-of-efficiency|Proof-of-Efficiency]] (CONCEPT)
- [[fitness-scorer|Fitness-Scorer]] (TOOL)
- [[genom|Genom]] (CONCEPT)
- [[forge-evolutions-engine|Forge (Evolutions-Engine)]] (SYSTEM)

## Relations
- Obolus-Watt-Ökonomie → PART_OF → Obulus
- Obulus → PART_OF → Miner (Agenten)
- Obulus → PART_OF → Validator (Richter)
- Obulus → PART_OF → Forge (Evolutions-Engine)
- Obulus → RELATED_TO → Obolus-Watt-Ökonomie
- Obulus → RELATED_TO → Proof-of-Efficiency
- Forge (Evolutions-Engine) → RELATED_TO → Genom
- Obulus → USES → Fitness-Scorer
