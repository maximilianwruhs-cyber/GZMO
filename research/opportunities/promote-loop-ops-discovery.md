---
id: promote-loop-ops-discovery
title: Promote-loop living apply for ops + discovery (after cognition soak)
status: candidate
score: 20
uniqueness: 5
brain_profit: 4
credit_cost: 3
attention_cost: 3
usp_fit: 5
stack_ids: [o2]
created: 2026-07-22
updated: 2026-07-22
---

# Promote-loop ops + discovery living apply

## Why rare

Knowledge + cognition living apply is pinned and waiting overnight soak. Ops/discovery stay **record-only** by design until that soak clears — then the same gated recipe can hand off the next narrow blast-radius loops.

## Doctrine

Protect the writer and the recipe; regenerate the vault. One writer (ADR-0003). `PROMOTE_ACK=1` + mutex + dual_writer_risk=false. Never whole-host cutover in this bet.

## Brain profit

Living host runs lab-beaten ops/discovery recipes overnight without expanding theater Unpark.

## Done when

1. [`promote-loop-living-apply`](promote-loop-living-apply.md) status=`soaked` (overnight Done when #4)
2. Explicit handoff recipe reviewed for `ops` and/or `discovery`
3. `PROMOTE_APPLY=1` lands at least one of those loops with per-loop pin + rollback
4. Brain Feed + keep-quality stay GREEN after one overnight post-apply

## Depends on

- [promote-loop-living-apply.md](promote-loop-living-apply.md) (active → soak)
- [beat-gate-versioned-baselines.md](beat-gate-versioned-baselines.md)

## Operator

```bash
bash scripts/promote-loop-soak-check.sh   # must be GREEN first
bash scripts/living-host-mutex.sh claim --host ct101 --note "promote-apply discovery"
PROMOTE_LOOP=discovery PROMOTE_ACK=1 PROMOTE_APPLY=1 bash scripts/promote-loop.sh
bash scripts/living-host-mutex.sh release
bash scripts/brain-feed-check.sh
```

## Sources

- [stack-future-opportunities-2026-07-21.md](../stack-future-opportunities-2026-07-21.md) O5 hard blocks
- Disposable-vault doctrine (flywheel end-of-craft 2026-07-22)
