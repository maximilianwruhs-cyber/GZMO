---
type: entity
title: "G12 — Eval green ≠ recall green"
created: "2026-06-26"
updated: "2026-06-26"
status: active
tags:
  - gap
  - discovery
  - recall
---

# G12 — Eval green ≠ recall green

Tracked KB feedback gap in `docs/DISCOVERY_KB_FEEDBACK_LOOP.md`. The discovery KB loop can pass its eval gate while recall quality is weak, because the recall smoke checks only for non-empty `gzmo memory search` output (hits at score 0.08 still count).

## Evidence

- `docs/CORE_MECHANICS_AUDIT_20260605.md`: strict recall 38/87 (0.437); faithfulness context FAIL (0.806).
- `data/loop-readiness-audit-2026-06-21.md`: KB Loop Closure scored 10/10 on a 5/5 binary smoke — masks weak semantic matches.
- All stored `recall-smoke-*.json` (2026-06-25/26) report pass_rate 1.0.

## Closure (thema_009)

[Verified Chain Recall](/entities/verified-chain-recall.md) adds a compositional probe reporting `atomic_pass_rate` and `chain_hit_rate` separately, extending THEMA_006 Pack H. A low initial `chain_hit_rate` is the expected signal that surfaces G12.
