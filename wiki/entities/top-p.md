---
type: entity
title: Top-P
created: 2026-06-08
updated: 2026-06-09
sources: 3
tags: []
status: draft
gzmo_synthetic: true
---



# Top-P

Type: CONCEPT

## From [drive-research-crafting-a-creative-agent-prompt](/entities/drive-research-crafting-a-creative-agent-prompt.md) (2026-06-08)
- Also known as Nucleus Sampling.
- Restricts model selection to a dynamic set of tokens whose cumulative probability mass equals the target value.
- A setting of 0.9 is considered optimal for a highly creative agent.

## From [drive-research-llamacpp-optimization-blueprint-micro03](/entities/drive-research-llamacpp-optimization-blueprint-micro03.md) (2026-06-09)
- A legacy sampler that calculates a cumulative probability mass and culls the tail.
- Must be disabled when using Min-P.

## From [optimizing-nvidia-blackwell-sm120-part1-micro02](/entities/optimizing-nvidia-blackwell-sm120-part1-micro02.md) (2026-06-09)
- Calculates a cumulative probability mass and culls the tail.
- Must be disabled when using Min-P.
