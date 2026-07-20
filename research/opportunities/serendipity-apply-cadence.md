---
id: serendipity-apply-cadence
title: Cheap serendipity promote cadence (no chat tourism)
status: soaked
score: 21
uniqueness: 4
brain_profit: 5
credit_cost: 5
attention_cost: 4
usp_fit: 4
stack_ids: [o5]
created: 2026-07-20
updated: 2026-07-20
---

# Serendipity apply cadence

## Why rare

Most RAG is similarity search. GZMO spark already produces verified-ish links; closing the loop back into the vault (promote) is uncommon and compounds honeypot mass.

## Brain profit

Dry-run candidates → occasional `SERENDIPITY_PROMOTE_APPLY=1` → living takeaway enqueue → distill → honeypot. Direct Brain Feed P0.

## Credit honesty

Sense/rank/dry-run/`serendipity-cadence.sh` need no Cursor chat. Apply is a 2-minute human gate after overnight spark.

## Done when

1. Weekly checklist in [BRAIN_FEED.md](../../docs/BRAIN_FEED.md) + `scripts/serendipity-cadence.sh`  
2. Artifact `data-next/serendipity/cadence-latest.json` (remind / ok / honest hold; **auto_apply=false**)  
3. `brain-feed-check.sh` includes cadence row and stays GREEN  

**Soaked 2026-07-20** — cadence script + checklist shipped. Human apply remains gated.

## Operator

```bash
bash scripts/serendipity-cadence.sh
# when candidates clear:
SERENDIPITY_PROMOTE_APPLY=1 bash scripts/serendipity-promote.sh
```
