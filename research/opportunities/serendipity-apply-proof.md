---
id: serendipity-apply-proof
title: Serendipity apply proof (close the 0-apply remind)
status: soaked
score: 20
uniqueness: 3
brain_profit: 5
credit_cost: 5
attention_cost: 3
usp_fit: 4
stack_ids: [o5]
created: 2026-07-20
updated: 2026-07-21
---

# Serendipity apply proof

## Why rare

Cadence + dry-run shipped; vault still barely gets spark links back because **human apply stays at zero**. Closing one verified apply turns fireworks into mass.

## Brain profit

1–3 clear dry-run candidates → `SERENDIPITY_PROMOTE_APPLY=1` → living takeaway enqueue → distill; cadence log shows recent apply.

## Done when

At least one successful apply logged in `cadence-log.jsonl` / promote artifact `applied[]`; brain-feed-check stays GREEN; no auto-apply default.

**Soaked 2026-07-21** — `scripts/serendipity-apply-proof.sh --apply` logged apply; cadence `applies_logged≥1`; `auto_apply=false`.

## Operator

```bash
bash scripts/serendipity-apply-proof.sh           # dry
bash scripts/serendipity-apply-proof.sh --apply   # human-gated ≤3
```
