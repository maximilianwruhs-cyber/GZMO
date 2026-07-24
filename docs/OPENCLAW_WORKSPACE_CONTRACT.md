# OpenClaw workspace ↔ GZMO ecosystem contract

**Status:** Active (2026-07-24)  
**USP:** nutrient · Brain Feed · airgap living — OpenClaw is an **operator surface**, not a second overnight brain  
**Doctrine:** [ADR-0003](./ADR-0003-one-instance-metabolism.md) · [ADR-0005](./ADR-0005-flywheel-over-frozen-topology.md) · [BRAIN_FEED.md](./BRAIN_FEED.md) · [EXTERNAL_LIVING_ATTACH.md](./EXTERNAL_LIVING_ATTACH.md) · [MACHINE.md](../MACHINE.md)

## One sentence

OpenClaw talks to Max on Telegram; **CT101 metabolizes**; the workstation **evolves**; living memory is reached only via **`gzmo-living`** + takeaway enqueue.

## Plane map

```text
┌───────────────────── OPERATOR (OpenClaw) ─────────────────────┐
│  Telegram / workspace ~/.openclaw/workspace                   │
│  Search: MCP gzmo-living    Nutrient: openclaw-takeaway.sh    │
│  Announce/orchestrate cron digests — does NOT own overnight   │
└───────────────┬───────────────────────────────┬───────────────┘
                │ read/search                   │ takeaway enqueue
                ▼                               ▼
┌──────────────── LIVING (CT101) ───────────────┐
│  gzmo-daemon · vault/honeypot · Redis/Qdrant/Neo4j            │
│  dream/distill/spark/ripen — ONE overnight writer             │
└───────────────────────────────────────────────────────────────┘
                ▲
                │ ops / research / opportunity (no second writer)
┌──────────────── EVOLVE (workstation) ─────────────────────────┐
│  systemd user timers: ops-health, research-scan, evolve-*     │
│  gzmo-serve ONLY if living-host-mutex claimed                 │
└───────────────────────────────────────────────────────────────┘
```

## File ownership in `~/.openclaw/workspace/`

| File | Owner | Sync? | Role |
|------|-------|-------|------|
| `ECOSYSTEM.md` | GZMO | **generated** | Plane map + never-list (single cheat sheet) |
| `LIVING_ATTACH.md` | GZMO | **generated** | Living MCP + takeaway contract |
| `GZMO_ECOSYSTEM_CRON.md` | GZMO | **generated** | Cron playbook |
| `CRON_JOBS.md` | GZMO sync | **generated snapshot** | Live OpenClaw + timer list |
| `TOOLS.md` | hybrid | ecosystem block **synced** | Host/SSH/MCP facts |
| `TOOLS.local.md` | OpenClaw | never overwrite | Cameras, TTS, personal nicknames |
| `AGENTS.md` | hybrid | ecosystem block **synced** | Rules; OpenClaw boilerplate kept |
| `SOUL.md` | hybrid | thin ecosystem boundaries synced | Persona stays human-shaped |
| `IDENTITY.md` | hybrid | name/role synced | Operator-surface identity |
| `USER.md` | hybrid | operator prefs synced | Max + GZMO working style |
| `HEARTBEAT.md` | OpenClaw | leave comments-only unless Max opts in | Empty = no heartbeat API spam |
| `memory/*.md` | OpenClaw | never | Daily scratch |
| `MEMORY.md` | OpenClaw | never | Curated long-term (main session) |
| `bin/*` | GZMO installers | synced/linked | `list-gzmo-crons`, `openclaw-takeaway` |

Markers in hybrid files:

```html
<!-- GZMO:ECOSYSTEM:BEGIN -->
…generated…
<!-- GZMO:ECOSYSTEM:END -->
```

Hand-edits **inside** markers are overwritten on next `sync-openclaw-workspace.sh`. Edit outside markers or use `TOOLS.local.md`.

## Never (all surfaces)

1. Second overnight writer (`gzmo-serve` while CT101 lives)  
2. Qdrant upsert / Neo4j auto-graph from chat  
3. `session close --now` while CT101 owns metabolism  
4. Claim “no cron jobs” without `bin/list-gzmo-crons.sh` / `CRON_JOBS.md`  
5. Treat OpenClaw as the GZMO product brain ([MACHINE.md](../MACHINE.md))

## Sync command

```bash
bash scripts/sync-openclaw-workspace.sh
# after install-openclaw-living-attach.sh / evolve timer changes
```

Optional: daily evolve already refreshes research/ops; call sync from `ecosystem-evolve-daily.sh` soft step.
