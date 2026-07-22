---
id: promote-loop-living-apply
title: Promote-loop living apply (gated handoff recipe)
status: active
score: 21
uniqueness: 5
brain_profit: 4
credit_cost: 3
attention_cost: 2
usp_fit: 5
stack_ids: [o2]
created: 2026-07-21
updated: 2026-07-22
---

# Promote-loop living apply

## Why rare

Record-only promote (`promote-loop.sh`) is soaked. The uniqueness craft needs a *reviewed* living apply path — still one writer, still `PROMOTE_ACK=1`, still mutex — without whole-host cutover.

## Doctrine (disposable vault)

Protect the writer and the recipe; regenerate the vault. Living apply lands the beaten **loop**, not vault rows.

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
bash scripts/living-host-mutex.sh claim --host ct101 --note "promote-apply cognition"
PROMOTE_LOOP=cognition PROMOTE_ACK=1 PROMOTE_APPLY=1 bash scripts/promote-loop.sh
bash scripts/living-host-mutex.sh release
bash scripts/brain-feed-check.sh
bash scripts/ct101-living-probe.sh
```

## Progress 2026-07-22

- `PROMOTE_APPLY` allowed for **knowledge + cognition** (`PROMOTE_APPLY_LOOPS`)
- Knowledge pinned `knowledge.v1` / `session-to-dream.sh` (09:09Z)
- Cognition pinned `cognition.v1` / `cognition-smoke.sh` (10:04Z)
- Per-loop pins: `living-applied-knowledge.json` + `living-applied-cognition.json` on CT101
- Post-apply: keep-quality GREEN, living-readiness GREEN, Brain Feed GREEN
- Serendipity week **2026-W30 = 3/3** capped applies done
- Nutrient transfer: 6 curated takeaways enqueued (no vault merge)
- **Soak remaining (Done when #4):** BF + keep-quality GREEN **after one overnight** calendar night
- Soak verifier: `bash scripts/promote-loop-soak-check.sh` → `data-next/beat-gate/promotions/soak-latest.{json,md}`
  (HOLD until min pin age ≥12h by default; then mark this bet `soaked`)
