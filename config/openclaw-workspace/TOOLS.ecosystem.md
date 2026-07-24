### Cron jobs (GZMO + OpenClaw)

**`cron` tool DENIED** — use exec / files. Refresh: `bash bin/list-gzmo-crons.sh` → updates via sync.

1. `CRON_JOBS.md` + `GZMO_ECOSYSTEM_CRON.md`
2. Never claim “no cron jobs” without those checks

### CT101 (living host)

- **SSH:** `ssh ct101` · IP `192.168.31.202` · Proxmox: `ssh pve "pct exec 101 -- …"`
- **Role:** overnight metabolism (`gzmo-daemon`) + Redis/Qdrant/Neo4j sidecars
- **Paths:** `/opt/gzmo/gzmo.toml`, `/opt/gzmo/data/`, `/opt/gzmo/current/`
- **Workstation:** evolve timers only — mutex before any `gzmo-serve`

### Living memory (gzmo-living MCP)

- Search: `gzmo_memory_search` / `gzmo_memory_status`
- Write nutrient: `bash bin/openclaw-takeaway.sh '…'`
- Playbook: `LIVING_ATTACH.md` · `ECOSYSTEM.md`

Personal/local nicknames → `TOOLS.local.md` (not overwritten by sync).
