---
type: entity
title: HGM
created: 2026-06-09
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---



# HGM

Type: SYSTEM

## From [ai-research-part3-micro02](/entities/ai-research-part3-micro02.md) (2026-06-09)
- Approximates the Gödel Machine by estimating Clade-Metaproductivity.
- Designed to promote higher weights to higher utility agents.
- Uses Thompson Sampling for probabilistic approximation of agent selection.

## From [ai-research-part8-micro04](/entities/ai-research-part8-micro04.md) (2026-06-09)
- Introduces Clade-Level Metaproductivity (CMP) as its guiding metric.
- Aggregates benchmark performances of an agent's entire subsequent lineage.
- Decouples the expansion phase from the evaluation phase.
- Utilizes Thompson Sampling populated by Beta distributions.
- Asynchronously samples agents for mutation based on clade-level promise.
- Utilizes rapid LLM-as-a-judge heuristics to rank candidate patches.
- Optimizes for long-term evolutionary potential rather than immediate reward.
