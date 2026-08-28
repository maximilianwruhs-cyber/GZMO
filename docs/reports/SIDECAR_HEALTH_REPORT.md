# Sidecar Health Check Report

**Date:** 2026-07-08 (corrected)

## CT101 (`gzmo health` via pct exec)

| Check | Status | Detail |
|-------|--------|--------|
| llm | OK (cloud) | `active_mode=cloud` → OpenRouter `deepseek/deepseek-v4-flash` (by design) |
| engine fallback | OK | `[engine.local]` → `http://192.168.31.184:8000/v1` (ornith-35b) for `cloud_first_background` failover |
| embeddings | OK | VM200 `:8081`, 1024 dims |
| qdrant | OK | honeypot → 24,083 points |
| rerank | OK | VM200 `:8081` router |
| redis | OK | localhost:6379 |
| neo4j | OK | bolt `192.168.31.202:7687` |
| mcp_memory | OK | 13,169 entities, 62,193 relations |
| gzmo-daemon.service | active | headless — **no Pi frontend on CT101** |
| Pi frontend | N/A | Runs on workstation — see [`PI_FRONTEND_SPLIT.md`](../ops/PI_FRONTEND_SPLIT.md) |

## Workstation (`after-boot-verify.sh`)

| Check | Status |
|-------|--------|
| Prime :8000 | OK — ornith-35b local |
| Pi KB embed :8002 | FAIL — service inactive |
| GZMO embed VM200 :8081 | OK |
| Rerank VM200 :8082 | FAIL — port not listening |
| Librarian VM200 :8083 | FAIL — port not listening |
| Qdrant knowledge/honeypot | OK |
| Local gzmo-daemon.service | inactive (expected — daemon on CT101) |
| Pi agent (`~/.pi/agent/`) | present — frontend for interactive use |

## Configuration notes

- **Do not** set CT101 `active_mode=local` unless intentionally switching daemon off cloud.
- `[engine.local].url` points at workstation Prime for background-task fallback only.
- Optional: deploy rerank `:8082` / librarian `:8083` on VM200; start `gzmo-embed.service` on workstation if Pi KB embed `:8002` needed.
