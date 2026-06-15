---
type: entity
title: Multi-Agent Reflexion (MAR)
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Multi-Agent Reflexion (MAR)

Type: FRAMEWORK

## From [[reflexion-framework-architectures-for-mitigating|reflexion-framework-architectures-for-mitigating]] (2026-06-08)
- Represents a critical milestone in the development of autonomous language agents.
- Demonstrates that carefully structured linguistic feedback can serve as a highly efficient, interpretable substitute for computationally expensive gradient-based weight updates.
- Maintains an episodic memory of its own failures.
- Achieved state-of-the-art results on MBPP and Leetcode Hard datasets.
- Combined with ReAct yielded an absolute 22% improvement over strong baseline approaches in ALFWorld.
- Agents continued to adapt while baseline ReAct-only agents plateaued.
- Operationalizes a novel paradigm known as verbal reinforcement learning.
- Updates an episodic memory buffer with linguistic feedback, allowing the model to self-correct in a few-shot manner.
- Requires highly specific prompt structures, deterministic external grounding, and rigorous memory management.
- Separates the execution of actions from evaluation and critique.
- Utilizes a tripartite architecture: Actor, Evaluator, and Self-Reflection model.
- Represents a continued evolution of the Reflexion paradigm.
- A structural evolution of the Reflexion paradigm.
- Replaces single-agent self-critique with a structured, adversarial debate among diverse, persona-based critics.
- Prevents repeated reinforcement of earlier mistakes by separating acting, diagnosing, and critiquing processes.
