---
type: entity
title: Curriculum Agent
created: 2026-06-08
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# Curriculum Agent

Type: CONCEPT

## From [[ai-research-part2|ai-research-part2]] (2026-06-08)
- Its training significantly impacts performance, with a 9.3% drop without it.
- Progressively generates more difficult tasks with the involvement of tools.
- Effectively generates increasingly difficult problems, from basic geometry to complex constraint satisfaction tasks.
- Training involves a Global Batch Size of 128.
- Uses a Learning Rate of 1× 10−6.
- Generates tasks with longer context dependencies and progressive difficulty.

## From [[ai-research-part8-micro04|ai-research-part8-micro04]] (2026-06-09)
- Is one of two specialized agents spawned from the same base LLM in Agent0.
- Leverages Ambiguity Dynamic Policy Optimization (ADPO) to procedurally generate tasks.
