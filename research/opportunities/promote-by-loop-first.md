---
id: promote-by-loop-first
title: First promote-by-loop (narrow cognition or knowledge)
status: soaked
score: 22
uniqueness: 5
brain_profit: 4
credit_cost: 4
attention_cost: 3
usp_fit: 5
stack_ids: [o2]
created: 2026-07-21
updated: 2026-07-21
---

# First promote-by-loop

## Why rare

ADR-0005 / LTL ADR-0003 allow narrow loop handoff after beat-gate PASS + operator ack. Doctrine exists; the product ritual is still unproven. Whole-host cutover still needs `CUTOVER_APPROVED=1`.

## Brain profit

Moves honest lab craft into the *current* living host without dual-writer chaos — flywheel over frozen topology.

## Done when

1. Versioned beat-gate baselines green for the chosen loop (depends on `beat-gate-versioned-baselines`)
2. `PROMOTE_LOOP=<name> PROMOTE_ACK=1 bash scripts/promote-loop.sh` writes a PASS record
3. `living-host-mutex.sh claim` around any living prove; dual_writer_risk≠true throughout
4. Documented promote decision under `data-next/beat-gate/promotions/`
5. Living **apply** still gated (PROMOTE_APPLY refused until handoff recipe reviewed)

**Progress 2026-07-21** — `scripts/promote-loop.sh` record-only path green for `knowledge` (gate_passed + knowledge.v1).

## Operator

```bash
bash scripts/living-host-mutex.sh status
bash scripts/beat-gate-kit.sh --loops cognition,knowledge
PROMOTE_LOOP=knowledge PROMOTE_ACK=1 bash scripts/promote-loop.sh
# living apply still refused — review data-next/beat-gate/promotions/latest.md
bash scripts/living-host-mutex.sh release
```

## Sources

- [stack-future-opportunities-2026-07-21.md](../stack-future-opportunities-2026-07-21.md) O5
- [ADR-0005-flywheel-over-frozen-topology.md](../../docs/ADR-0005-flywheel-over-frozen-topology.md)

**Soaked 2026-07-21** — record-only ritual green (`promote-loop.sh` knowledge + knowledge.v1). Living `PROMOTE_APPLY` deferred to a follow-up bet.
