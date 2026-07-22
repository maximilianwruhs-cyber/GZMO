# Arena → Pin demo (theater)

**Status:** Demable Unpark Wave 3 — **not** Brain Feed GREEN claim, **not** living toml merge  
**Front door:** `bash scripts/arena-pin-demo.sh`  
**Boundary:** [OBOLUS_ARENA_BOUNDARY.md](./OBOLUS_ARENA_BOUNDARY.md) · doctrine: [BRAIN_FEED.md](./BRAIN_FEED.md) P1

## What you feel

Nightburst **champion suggestion** → Forge **recommend pins** → Brain intel **human pin ritual** + pin-log (accept/reject/defer). Living `/opt/gzmo/gzmo.toml` stays untouched. Daemon Arena jobs stay off.

```bash
bash scripts/arena-pin-demo.sh
bash scripts/arena-pin-check.sh
# Optional fresh burst (heavier):
ARENA_FORCE_RUN=1 bash scripts/arena-night-check.sh
```

## Chain

| Step | Script | Output |
|------|--------|--------|
| Suggest | `arena-night-check.sh` | `data-next/arena/champion-suggestion.toml` |
| Observe | `arena-lab-demo.sh` | RAPL / €/night (soft) |
| Recommend | `forge-lab-demo.sh` | `data-next/forge-lab/recommend.json` (`blocks_distill=false`) |
| Ritual | `brain-intel-promote.sh` | `data-next/brain-intel/living-pin-suggestion.md` |
| Decide (log) | `brain-intel-pin-log.sh` | `data-next/brain-intel/pins/` + `pin-log-latest.json` |

## Hard rules

1. **Suggest-only** — never `auto_apply` champion into living config from this path.  
2. **No daemon Arena jobs** by default — sibling `obolus-arena/` owns overnight z-loops.  
3. Pin-log `accept` is a **decision record**, not a merge. Human still edits living toml on CT101 by hand.  
4. Estimate joules ≠ RAPL trust until probe/ACL says so.

## Artifacts

- `data-next/arena-pin/demo.json` — chain inventory  
- `data-next/arena-pin/felt-latest.md` — human-readable champion + pin status  
- `data-next/arena-pin/latest.json` — check verdict  

See [STACK_OPPORTUNITY_MAP.md](./STACK_OPPORTUNITY_MAP.md) Energy/Arena · [UNPARK_ROADMAP.md](./UNPARK_ROADMAP.md) Wave 3.
