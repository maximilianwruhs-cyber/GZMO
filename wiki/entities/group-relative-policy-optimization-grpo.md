---
type: entity
title: Group Relative Policy Optimization (GRPO)
created: 2026-06-08
updated: 2026-06-09
sources: 5
tags: []
status: draft
gzmo_synthetic: true
---






# Group Relative Policy Optimization (GRPO)

Type: CONCEPT

## From [[ai-research-part2|ai-research-part2]] (2026-06-08)
- Original algorithm does not account for the reliability of pseudo-labels.
- Training the execution agent using the original GRPO with standard advantage and clipping resulted in a performance drop of 1.9%.
- A reinforcement learning method that avoids training a critic by using intra-group relative rewards.
- Used to update the policy in Agent0.
- The Curriculum Agent is optimized using this algorithm.

## From [[ai-research-part8-micro05|ai-research-part8-micro05]] (2026-06-09)
- An algorithm implementable in verl.

## From [[ai-research-part8-micro08|ai-research-part8-micro08]] (2026-06-09)
- A training method in TRL.
- For transformer language models.

## From [[drive-research-agentic-token-economy-blueprint-micro02|drive-research-agentic-token-economy-blueprint-micro02]] (2026-06-09)
- Advanced RL pipelines build heavily upon GRPO.
- Traditional RLHF optimizes for human preference, often equating to verbosity.
- Token-economic RL optimizes for precision, brevity, and strategic abstention.
- BACR introduces Budget-Conditioned Advantage Estimation (BCAE) which conditions the advantage baseline specifically on the allocated budget level using a lightweight value function.

## From [[the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro06|the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro06]] (2026-06-09)
- Technique used by OpenClaw-RL for policy training.
- Updates model weights.
