# Ten organism leaps — craft progress 2026-07-22

Shipped same-day craft for G1–G10 + research/borrow-eval. Wall-clock soaks (G2 honest night 3, promote pin ≥12h) run via watchers — never faked.

## Landed

| Leap | Artifact |
|------|----------|
| Research | [sleep-consolidation-sota-2026-07-22.md](sleep-consolidation-sota-2026-07-22.md) |
| G3 | `daemon_cmd.rs` schedules promote+embed (`spawn_blocking`) with metabolism cron |
| G4 | honeypot `utility_score` schema v8 (+ **v9 repair** if column missing); `reinforce_by` bumps utility; search ORDER BY utility |
| G5 | `gzmo immune forget` + `IMMUNE_APPLY=1 gzmo immune apply` + value-forgetting plan |
| G6 | `scripts/sleep-time-budget.sh`, `scripts/airgap-overnight-soak.sh` |
| G7 | `opportunity-sense.sh` scars: organ-trace, promote soak, sleep budget |
| G8 | ops-discovery bet weekly habit checklist |
| G9 | airgap install smoke + mutex appliance hosts |
| G10 | `scripts/organism-surface.sh` |
| borrow-eval | `scripts/organism-memory-bench-spike.sh` |
| G2 watchers | `promote-loop-soak-watch.sh`, `honest-soak-night-watch.sh` |

## Still wall-clock

- Promote soak GREEN when min pin age ≥12h (~22:04Z 2026-07-22) — watcher running
- Honest nights 3/3 after ≥18h from last GREEN (~06:40Z 2026-07-23) — watcher running

## Operator

```bash
bash scripts/promote-loop-soak-watch.sh
bash scripts/honest-soak-night-watch.sh
bash scripts/organism-surface.sh
bash scripts/organism-memory-bench-spike.sh
```
