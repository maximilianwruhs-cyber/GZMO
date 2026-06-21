---
type: entity
title: Loop Engineering
created: 2026-06-21
updated: 2026-06-21
sources: 4
tags: ["thema_006", "hypervisor", "closed-loop"]
status: draft
gzmo_synthetic: true
---

# Loop Engineering

Type: CONCEPT / SYSTEM

Loop Engineering is the design paradigm that moves agentic workflows from human-in-the-loop manual prompting to automated, closed-loop execution. It enforces deterministic system boundaries around probabilistic models to guarantee safe, reliable, and cost-bounded operations.

## Core Mechanisms (The Four Pillars)

1. **State Space Perception & Observation Filters**: Translating raw stdout/stderr logs into actionable summaries. Leverages semantic compaction strategies like [compaction](/entities/compaction.md) to mitigate "context rot."
2. **Agentic Decision & Action Trajectories**: Restricting tool usage via rigid schemas and standardized Model Context Protocols (MCP).
3. **Deterministic Verification Gates**: Programmatic tests running inside isolated sandboxes to verify work outcomes.
4. **Shared Blackboard State Architectures**: Decoupling agent conversation logs from system state using SQLite databases or vector DB backends.

## Key Mitigation Strategies & Cross-Links

- **Poisoned Blackboard**: Mitigated via [git-worktree](/entities/git-worktree.md) workspace isolation, preventing recovery agents from inheriting contaminated working directories.
- **Blind Spinning / Runway Loops**: Prevented by the `StuckDetector` and [circuit-breakers](/entities/circuit-breakers.md) that abort execution upon detecting repetitive patterns or ping-pong oscillations.
- **Resource Exhaustion**: Managed via [token-buckets](/entities/token-buckets.md) and integral-depletion budgets restricting execution costs.

## Historical Lineage
1. **Single-Turn Imperative Prompting (2022–2024)**: Text-to-text optimization with heavy human review.
2. **ReAct Exploratory Paradigm (2023–2024)**: Alternating thought-action loops prone to context rot.
3. **Harness & Context Engineering (2025)**: Standardizing rule profiles like `CLAUDE.md`.
4. **Autonomous Systemic Loop Engineering (2026)**: Unattended background loops driven by multi-agent maker-checker setups.
