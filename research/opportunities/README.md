# Opportunity bet log

Operator research bets for GZMO upgrades. Doctrine: [docs/OPPORTUNITY_DISCOVERY.md](../../docs/OPPORTUNITY_DISCOVERY.md).

## Status values

| Status | Meaning |
|--------|---------|
| `candidate` | Sensed / ranked, not locked |
| `active` | Exactly one preferred — current ship mission |
| `soaked` | Shipped; keep-quality / brain-feed still healthy |
| `killed` | Rejected (low score, USP miss, or failed soak) |
| `horizon` | Parked until world changes (e.g. local 128k+ on 32GB) |

## Index

| Id | Score | Status | Title |
|----|-------|--------|-------|
| [tinyfolder-living-one-shot](tinyfolder-living-one-shot.md) | 20 | **active** | One-command tinyFolder → living takeaway enqueue |
| [takeaway-side-effect](takeaway-side-effect.md) | 20 | soaked | Takeaways only as side-effect of real work |
| [serendipity-apply-cadence](serendipity-apply-cadence.md) | 21 | soaked | Cheap serendipity promote cadence (no chat tourism) |
| [opportunity-discovery-cycle](opportunity-discovery-cycle.md) | 22 | soaked | This cycle (Sense→Rank→Bet→Ship) |
| [local-intel-32gb-128k](local-intel-32gb-128k.md) | — | horizon | Local strong model + long context on 32GB VRAM |

## Schema (YAML frontmatter)

```yaml
---
id: kebab-id
title: Short title
status: candidate|active|soaked|killed|horizon
score: 0-25          # omit for horizon
uniqueness: 0-5
brain_profit: 0-5
credit_cost: 0-5     # higher = cheaper
attention_cost: 0-5  # higher = less babysitting
usp_fit: 0-5
stack_ids: [o5, m3]  # optional STACK_OPPORTUNITY_MAP ids
created: YYYY-MM-DD
updated: YYYY-MM-DD
---
```
