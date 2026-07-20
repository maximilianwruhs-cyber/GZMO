---
id: living-organ-trace
title: Living organ-trace (CT101 scheduler-runs, not workstation zoo)
status: candidate
score: 20
uniqueness: 4
brain_profit: 3
credit_cost: 5
attention_cost: 4
usp_fit: 4
stack_ids: [o1]
created: 2026-07-20
updated: 2026-07-20
---

# Living organ-trace

## Why rare

“Which organs fired overnight” is rare when tied to the **living** vault. Current organ-trace tends to read workstation `data-next/scheduler-runs`.

## Brain profit

Operator sees real living metabolism (distill/dream/spark/…) without starting a second writer or Observatory theater.

## Done when

`organ-trace` (or flag) pulls `/opt/gzmo/data/scheduler-runs` via SSH; `latest.json` lists living jobs; soft note if distill/dream missed — never dual-writer.
