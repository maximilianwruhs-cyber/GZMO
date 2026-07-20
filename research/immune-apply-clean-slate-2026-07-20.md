# Immune apply — clean-slate engine lore (2026-07-20)

Operator-approved supersession on CT101 living vault after living-quality research flagged contradictory “DreamEngine/Spark/SessionDistill currently disabled during clean-slate rebuild” facts while overnight metabolism was clearly running.

## Applied (dry_run → apply)

| Wave | Superseded | Replacement |
|------|------------|-------------|
| DreamEngine | 3 ids | `71f73cbd-…` Enabled on CT101 |
| SparkEngine (+ mop) | 4 ids | `a269545e-…` Enabled |
| SessionDistill (+ mop) | 8 ids | `564ca341-…` Enabled |
| STATE:EnginesDisabled | 1 id | `778d2cbe-…` EnginesEnabled |

Artifacts on CT101:

- `/opt/gzmo/data/immune/applied-2026-07-20.json`
- `/opt/gzmo/data/immune/applied-2026-07-20-spark-session.json`
- `/opt/gzmo/data/immune/applied-2026-07-20-mop.json`
- `/opt/gzmo/data/immune/latest-apply.json`

## Follow-up code

Immune patrol now treats Dream / Spark / SessionDistill / `[STATE:EnginesDisabled]` clean-slate disabled claims as one **global** class, and **exempts** Enabled replacements + Legacy auto_dream ops notes so re-patrol stays at 0 after apply.

## Qdrant

Honeypot recall filters `is_latest=1` in SQLite after ID fetch; superseded rows should not surface. Optional point delete of superseded UUIDs keeps collection cardinality honest (`scripts/qdrant-delete-ids.py` when present).
