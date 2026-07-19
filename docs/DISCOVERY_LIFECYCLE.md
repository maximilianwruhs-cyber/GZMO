# Discovery Lifecycle — Scout vs Implement

**Status:** Doctrine recovered from Cursor plan `discovery_redesign_approach_*` (2026-07-19)  
**Living wire (timers/paths):** [DISCOVERY_LIVING_WIRE.md](./DISCOVERY_LIVING_WIRE.md)  
**KB loop:** [DISCOVERY_KB_FEEDBACK_LOOP.md](./DISCOVERY_KB_FEEDBACK_LOOP.md)  
**Paths:** [CT101_PATH_AUTHORITY.md](./CT101_PATH_AUTHORITY.md)

## Default product shape

**Discover-only scout** on a short tick. **Implementation** only from human-approved or separately scheduled drain of **published** reports.

| Layer | Job | Success |
|-------|-----|---------|
| Forum-1 scout | Probe → teach → cycle report | Report on disk + teach ≥1 |
| Publish seam | Actionability gate on **cycle reports** | `latest.md` + optional enqueue |
| Forum-2 implement | Drain queue **outside** discovery flock | Plan fidelity + real diffs |

Default: `DISCOVERY_INLINE_IMPLEMENT=0` (discover only, enqueue). Do not treat 60‑minute session finals as the health KPI.

## Why the maximal loop kept failing

1. **Timer = session lifecycle** — `stop_discovery_timer()` after session final → autopilot dies on purpose and must re-arm.  
2. **One `state.json` mutex** — ghost `completed_unpublished` / short sessions re-enter session-final and kill the real run.  
3. **Publish bar vs cheap scout models** — cycle reports may publish; session finals often fail actionability → unpublished streaks.  
4. **Discover ↔ implement coupled** — `INLINE=1` runs remediator under discovery lock/timeouts; drain retries dead plans.  
5. **Mentor socket is not the real dependency** — OpenRouter teach fallback exists; preflight must not require socket forever.

## Working signature

- Timer stays armed (or chain auto-rearms) without babysitting  
- Every tick: teach gate + cycle report on disk  
- Published cycles update `latest.md`; enqueue only on publish  
- Implement when **you** drain — not mid-dialogue  
- No ghost sessions disabling the timer  

Healthy metrics: see [DISCOVERY_LIVING_WIRE.md](./DISCOVERY_LIVING_WIRE.md) (`lock_wait` ok; frequent `lock_skip` = stale deploy).

## Ops first moves

```bash
# On CT101 discovery config.env (gzmo_skills):
# DISCOVERY_INLINE_IMPLEMENT=0
# Prefer small ticks + AUTO_CHAIN / multi-cycle over 60m finals as KPI
```

1. Confirm discover-only + timer stays up across ticks.  
2. Archive stuck queue rows with `plan_fidelity_score=0`.  
3. Only then enable a **scheduled** drain (not inline).

## Deliberately declined

- Inline implement as primary path  
- More patches to make 60m finals the health metric  
- Native mentor socket as a hard blocker for Forum-1 scout
