# Living organs — CT101 proof (2026-07-20)

Post-merge deploy of [PR #76](https://github.com/maximilianwruhs-cyber/GZMO/pull/76) onto `/opt/gzmo/current` (survey_GZMO tree). Daemon restarted after `chmod +x` on `gate-pre-deploy.sh`.

## Felt Use — **VERIFIED**

| | latest | nonzero `recall_count` | max |
|--|--:|--:|--:|
| Before search panel | 38 730 | **0** | 0 |
| After ~11 searches | 38 730 | **55** | **3** (Cited weight) |

Hits include honeypot/pipeline/SparkEngine facts with `recall_count=3`.

## Refractory Field — **VERIFIED**

Two manual `gzmo spark` cycles:

1. Anchor `[SYSTEM:pi-coding-agent]` Telegram-adapter (not the old four-layer GZMO monoculture).
2. Anchor `[TOOL:Librarian]` VM200 librarian — **different** community; `refractory_entries` went 1 → 2.

Artifacts: `/opt/gzmo/data/spark/refractory.json`, `last-spark-report.json`.

## Night Lymph — **VERIFIED**

`/opt/gzmo/data/night-lymph/latest.json` — `night_id=2026-07-20`, `sparks=2`.

## Immune Patrol — **VERIFIED**

`gzmo immune plan` → `/opt/gzmo/data/immune/plan-2026-07-20.json` — **4** dry_run candidates, including:

- `[SYSTEM:DreamEngine] Currently disabled during clean-slate rebuild` (×2 variants)
- Legacy auto_dream disabled lore
- One false-positive-ish Dream schedule fact (refine needles later; plan-only so safe)

## Faithfulness widen — **VERIFIED**

`faithfulness-living.sh` with 12 claims: **12/12** `living_ok`.

## Deploy scars

- First rsync missed `memory/felt_use.rs` / vault method updates — force-sync `gzmo-core/src/`.
- `gate-pre-deploy.sh` lost `+x` → systemd 203/EXEC until `chmod +x`.
