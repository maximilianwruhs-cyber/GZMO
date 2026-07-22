---
id: serendipity-weekly-apply
title: Serendipity capped weekly apply habit
status: soaked
score: 20
uniqueness: 4
brain_profit: 4
credit_cost: 5
attention_cost: 4
usp_fit: 5
stack_ids: [o2]
created: 2026-07-22
updated: 2026-07-22
---

# Serendipity capped weekly apply habit

## Habit bet
Weekly human-gated apply ≤3/ISO-week; horizon/local-intel filtered out of living takeaways.

## Done when
- [x] `SERENDIPITY_WEEKLY_CAP=3` enforced via `weekly-apply-log.jsonl`
- [x] Horizon filter drops TurboQuant / 256K-on-32GB theater
- [x] Human apply enqueued on living host (no dual-writer, no auto-apply)
- [x] `bash scripts/serendipity-weekly-check.sh` OK
- [x] Brain Feed GREEN

## Soak evidence (2026-07-22)
- Dry-run: candidates=3 after filter, `filtered_out=2` (TurboQuant ×2)
- Apply: session `serendipity-promote-fea64625`, week **2026-W30 applies=1/3** (Zellij + pi-telegram + GZMO)
- Weekly check: `serendipity_weekly_ok — 1/3 applies in 2026-W30`
- Brain Feed: verdict GREEN

```bash
bash scripts/serendipity-promote.sh                    # dry-run + USP filter
SERENDIPITY_PROMOTE_APPLY=1 bash scripts/serendipity-promote.sh
bash scripts/serendipity-weekly-check.sh
bash scripts/brain-feed-check.sh
```
