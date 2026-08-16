# Agentic memory + harness + local-LLM — research kickoff

Copy everything below the line into a **new** agent chat. Do not babysit with “continue.”  
The atlas already exists. This sitting **grafts one loop** or writes one candidate bet — it does not re-survey the field.

---

## Mission

**Product:** a living Keep — honeypot + verify + promote on one airgapped box. Frozen local models get better from three planes: **memory metabolism**, **harness**, **context/playbook lift** — not overnight LoRA.

**Read first (do not rewrite):** [`research/agentic-memory-harness-local-llm-2026-08-16.md`](../../research/agentic-memory-harness-local-llm-2026-08-16.md)

**Active ship (exactly one):** [`felt-use-mass-growth`](../../research/opportunities/felt-use-mass-growth.md). Rank 0 in the atlas is living mass + soak nights. Open after M3/H1/M7/L1/T1: immune apply on living (lab until soak).

**Not the product:** Observatory, OKForge, HSP, pantheon, AOS CE, `eml-core`, Mem0/Letta SKU, public MCP HTTP.

**Done when (falsifiable):**

1. CT101 census attempted: `bash scripts/felt-use-depth.sh` (SSH fail → `INCONCLUSIVE`/`RED`, never synthetic 0=GREEN).
2. Either (a) one PR that moves felt-use/utility **or** lands immune apply on living after soak, or (b) a candidate bet file with score ≥18, `brain_profit≥3`, `usp_fit≥4`, status `candidate`.
3. Explicit park list in the closer (glass, SKU, LoRA, second writer).

---

## Already decided (do not re-litigate)

| Decision | Meaning |
|----------|---------|
| Three planes, one animal | Memory / harness / frozen-model lift. All grafts land on organs in `gzmo-core`. |
| One writer | ADR-0003 / mutex. Telescope never overnight. |
| #166/#167 shipped | Q-select, Outcome Q, region rewrite, `gate_event`, `failure_cases` write+bounded retrieve. Do not re-ship. |
| ACE is curator deltas | Incremental ADD/UPDATE/REMOVE under the gate or a **human pin**. No unsupervised SOUL rewrite. |
| Skills ≠ vault | Cursor grill/tdd/handoff help the operator. Vault mass requires takeaway / MCP felt-use. |
| Harvest ≠ SKU | Steal retrieve/forget/supersede/consolidate/verify/time/prune/pin. Reject SaaS, multi-tenant HTTP, overnight LoRA. |

---

## Organs (map every steal here)

| Organ | Plane | Honest evolve-toward |
|-------|-------|----------------------|
| Distill / gate / honeypot | 1 | Failure-case retrieve in-tree; typed refuse already written |
| Recall / felt-use | 1 | Living Q mass from real sessions |
| Dream / immune / spark | 1 | Region rewrite in-tree; forget apply still lab; spark stays verify-then-promote |
| `agent_loop` / `context.rs` | 2 | Workflow skill pin in-tree |
| `workflow_skills` | 2+3 | ACE deltas on one `SKILL.md`, human-pinned |
| MCP / daemon | 2 | Attach HOLD; one writer |
| Arena / calibration | 3 | Suggestion → human pin only |

---

## Method

```text
Read the 2026-08-16 atlas (do not re-tour arXiv)
  → living census (felt-use-depth)
    → pick rank 0 if mass is thin, else at most one open loop
      → cargo test / clippy
        → commit → push → PR → CI green
```

```bash
cd /home/mw/gzmo_full
export TMPDIR=/home/mw/.cache/tmp
bash scripts/felt-use-depth.sh
bash scripts/brain-feed-check.sh
# LIVING_GATE_SKIP_TAKEAWAY=1 bash scripts/keep-quality-gate.sh

cargo test
cargo clippy --all-targets -- -D warnings
```

Forbidden: `gzmo serve` on the telescope; publicizing herdr/okforge; new crates; flipping the active bet; Observatory GREEN; overnight LoRA; memory-gym chats.

## Brutal test

If the morning vault and the next MCP attach would be unchanged, you wrote costume. Stop.
