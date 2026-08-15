# OpenClaw ↔ GZMO living attach

**Generated** — `scripts/sync-openclaw-workspace.sh`  
**Server:** `gzmo-living` via `scripts/pi-gzmo-mcp-serve.sh` (CT101 `/opt/gzmo`)

## Search (read)

MCP tools: `gzmo_memory_search`, `gzmo_memory_status`, `gzmo_memory_profile`, `gzmo_wiki_search`, …

```bash
openclaw mcp show gzmo-living
bash ~/github-clone/GZMO/scripts/living-attach-check.sh
```

## Nutrient write (enqueue only)

```bash
bash bin/openclaw-takeaway.sh 'durable fact for living distill'
# → CT101 session close --takeaway, no --now, dual-writer refuse
```

## Never

- Qdrant upsert into `honeypot`
- Neo4j auto-graph from Telegram
- `systemctl --user start gzmo-serve` while CT101 lives
- `GZMO_PRODUCT=1` / `GZMO_ALLOW_LAB_VAULT=1` on this bridge

## Docs

GZMO: `docs/EXTERNAL_LIVING_ATTACH.md` · `docs/BRAIN_FEED.md` · `docs/OPENCLAW_WORKSPACE_CONTRACT.md`