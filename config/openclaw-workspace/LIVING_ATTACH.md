# OpenClaw ↔ GZMO living attach

**Generated** — `scripts/sync-openclaw-workspace.sh`  
**Server:** `gzmo-living` via `scripts/pi-gzmo-mcp-serve.sh`  
**Living host:** **workstation** (`gzmo-daemon`, `~/.gzmo/`) — CT101 decommissioned (2026-10-06 doctrine)

## Search (read)

MCP tools: `gzmo_memory_search`, `gzmo_memory_status`, `gzmo_memory_profile`, `gzmo_wiki_search`, …

```bash
openclaw mcp show gzmo-living
bash ~/github-clone/GZMO/scripts/living-attach-check.sh
```

## Nutrient write (enqueue only)

```bash
bash bin/openclaw-takeaway.sh durable fact for living distill
# → living session close --takeaway, no --now, dual-writer refuse
```

## Never

- Qdrant upsert into `honeypot`
- Neo4j auto-graph from Telegram
- `systemctl --user start gzmo-serve` while living daemon already owns overnight
- `GZMO_PRODUCT=1` / `GZMO_ALLOW_LAB_VAULT=1` on this bridge
- SSH `ct101` / `192.168.31.202` as if the vault still lives there

## Docs

GZMO: `docs/EXTERNAL_LIVING_ATTACH.md` · `docs/BRAIN_FEED.md` · `docs/OPENCLAW_WORKSPACE_CONTRACT.md`
