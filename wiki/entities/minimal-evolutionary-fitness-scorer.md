---
type: entity
title: Minimal Evolutionary Fitness Scorer
created: 2026-06-09
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# Minimal Evolutionary Fitness Scorer

Type: TOOL

## From [[obolus-micro03|obolus-micro03]] (2026-06-09)
- Final version with fixes integrated.
- Config-driven, energy-based efficiency, explicit error handling.
- Includes dataclasses for TrialResult and ScoringConfig.
- Functions for computing quality, efficiency, variance penalty, and z-score.
- Provides decision logic for approving mutations.

## From [[the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro08|the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro08]] (2026-06-09)
- A Python script for evaluating mutations.
- Config-driven and robust.
- Focuses on Quality, Efficiency, and Variance.
