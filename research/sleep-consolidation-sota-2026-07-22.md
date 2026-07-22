# Sleep-consolidation SOTA → GZMO organism (2026-07-22)

Research capture for the ten-leap organism plan. **Borrow algorithms and snippets; do not clone Mem* SaaS or dual overnight writers.**

## Field consensus

Dreaming = offline memory curation (merge, forget, rewrite). Wake executes; sleep consolidates. GZMO already has distill→promote→embed→dream/spark; gaps are utility-weighted retrieval, intentional forgetting, when-to-dream policy, and external organism metrics.

## Cite list (steal column = GZMO leap)

| Work | Link | License / notes | Steal | Leap |
|------|------|-----------------|-------|------|
| CoALA | https://arxiv.org/abs/2309.02427 | paper | memory kinds + action taxonomy | G10 |
| Generative Agents | https://arxiv.org/abs/2304.03442 | paper | reflection → semantic memory | G10 |
| MemGPT / Letta | https://arxiv.org/abs/2310.08560 | Apache-2 (Letta) | tier paging / eviction policy | G3–G4 |
| Sleep-time Compute | https://arxiv.org/abs/2504.13171 | paper | when-to-dream budget | G6 |
| CLS hippocampal–cortical | https://doi.org/10.1101/2022.01.31.478475 | paper | vault=hippocampus, honeypot=cortex | G3 |
| CLS toy | https://github.com/hadbierox196/Implementation-of-hippocampal-cortical-interaction | check repo LICENSE | replay teacher intuition | G3 |
| Mem0 | https://arxiv.org/abs/2504.19413 | Apache-2 (OSS) | extract→consolidate→retrieve; graph for relations | G4 |
| MemoryOS | https://arxiv.org/abs/2506.06326 · https://github.com/BAI-LAB/MemoryOS | check LICENSE | STM→MTM→LPM heat promote | G3 |
| SCM | https://arxiv.org/html/2604.20943v1 | paper | NREM/REM + value forgetting + self-model | G5 |
| MyGO | https://arxiv.org/html/2508.21296v1 | paper | generative dream data (horizon) | horizon |
| MemRL | https://arxiv.org/html/2601.03192v2 | paper | Intent–Experience–Utility; two-phase retrieval | G4 |
| Memento | https://arxiv.org/abs/2508.16153 · https://github.com/Agent-on-the-Fly/Memento | check LICENSE | memory MDP / case rewrite | G4 |
| Memento 2 | https://arxiv.org/pdf/2512.22716 | paper | read=improve, write=evaluate | G4–G7 |
| Phasor Agents | https://arxiv.org/pdf/2601.04362 | OSS (check) | wake tag / offline capture staging | G5–G6 |
| Anthropic Dreams lineage | https://ogham-mcp.dev/blog/memory-consolidation-lineage/ | blog | immutable raw → curated output store | G5 |
| LoCoMo | https://arxiv.org/abs/2402.17753 · https://github.com/snap-research/locomo | research | satellite QA bench | borrow-eval |
| LongMemEval | Wu et al. 2024 | research | harder multi-session bench | borrow-eval |

## Explicit non-goals (living box)

- Overnight LoRA / EWC weight updates (HAL sleep-wake, techRxiv cyclical PEFT)
- Public multi-tenant Mem* SKU
- Second overnight writer (ADR-0003)
- Cloud-required core metabolism (ADR-0004)

## Borrow-next ranking (same-day craft)

1. MemRL-style `utility_score` on honeypot + reinforce bump (G4)
2. Daemon-scheduled promote+embed with `write_job_run` (G3)
3. SCM-style value forgetting + capped immune apply (G5)
4. Sleep-time budget from nutrient backlog (G6)
5. Thin LoCoMo/LongMemEval spike harness (borrow-eval)

## Attribution

When porting snippets from Apache-2/MIT/BSD repos, keep LICENSE notice in file headers or this note. Prefer reimplementation in `gzmo-core` over vendoring whole stacks.
