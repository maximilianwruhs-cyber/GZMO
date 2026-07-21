---
id: promote-loop-living-apply
title: Promote-loop living apply (gated handoff recipe)
status: candidate
score: 21
uniqueness: 5
brain_profit: 4
credit_cost: 3
attention_cost: 2
usp_fit: 5
stack_ids: [o2]
created: 2026-07-21
updated: 2026-07-21
---

# Promote-loop living apply

## Why rare

Record-only promote (`promote-loop.sh`) is soaked. The uniqueness craft needs a *reviewed* living apply path — still one writer, still `PROMOTE_ACK=1`, still mutex — without whole-host cutover.

## Brain profit

Moves a proven loop into the current living host so overnight metabolism runs lab-beaten recipes.

## Done when

1. Explicit handoff recipe per loop (knowledge or cognition) reviewed in docs
2. `PROMOTE_APPLY=1` only after mutex claim + dual_writer_risk=false
3. Rollback note + promote artifact under `data-next/beat-gate/promotions/`
4. Keep-quality / Brain Feed stay GREEN after one overnight

## Depends on

- [promote-by-loop-first.md](promote-by-loop-first.md) (soaked record ritual)
- [beat-gate-versioned-baselines.md](beat-gate-versioned-baselines.md)

## Operator

```bash
bash scripts/living-host-mutex.sh claim --host workstation --note "promote-apply knowledge"
PROMOTE_LOOP=knowledge PROMOTE_ACK=1 PROMOTE_APPLY=1 bash scripts/promote-loop.sh  # after recipe ships
bash scripts/living-host-mutex.sh release
```
