---
id: overnight-metabolism-triad
title: Overnight metabolism triad (TinyFolder glue + daemon ledger + spark dampening)
status: soaked
score: 22
uniqueness: 4
brain_profit: 5
credit_cost: 4
attention_cost: 3
usp_fit: 5
stack_ids: [o1, m4, r5]
created: 2026-07-21
updated: 2026-07-21
---

# Overnight metabolism triad

## Why

2026-07-20/21 CT101 night: spark correctly discovered TinyFolder→HTTP/CLI-free ingest, then re-hypothesized it ~90×; `DREAMS.md` grew while lymph/ledger stayed blind (`scheduler-runs` only watchdog).

## Done when

1. `tinyfolder-overnight.sh` enqueues pending drops to living distill without CLI / `--now`
2. `gzmo daemon` writes `scheduler-runs/latest-{dream,spark,distill}.json` + watchdog tick
3. Spark tag/theme refractory stronger (defaults 120h / 0.95; skip path updates lymph)

**Soaked 2026-07-21** — code+scripts on main; living needs CT101 binary rebuild/restart for ledger+refractory.

## Operator

```bash
bash scripts/tinyfolder-overnight.sh --dry-run
# binary deploy: docs/CT101_DEPLOY.md §Deploying a new binary
```
