---
id: soak-night-honest-timer
title: Honest soak timer (min spacing between GREEN samples)
status: active
score: 19
uniqueness: 2
brain_profit: 3
credit_cost: 5
attention_cost: 4
usp_fit: 5
stack_ids: []
created: 2026-07-20
updated: 2026-07-21
---

# Soak night honest timer

## Why rare

Commodity — but USP claims need it. Soak can currently look ready from same-hour GREEN samples; nights are the real bar.

## Brain profit

Indirect: stops false confidence that living quality is soaked when samples are minutes apart.

## Done when

`keep-quality-soak.sh --summary` requires min inter-sample spacing (e.g. ≥18h) or CT101 appends one sample/night; same-hour streaks → HOLD not `soak_ready_unpark_ok`.
