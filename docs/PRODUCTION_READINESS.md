# Production readiness checklist

**Authority:** [gzmo_placement_architecture.md](./gzmo_placement_architecture.md)

## One command

```bash
cd ~/Projects/_foundation-audit/survey_GZMO
./scripts/p1-readiness-test.sh
```

Exit `0` = P1 quality + production E2E pass.

**Stack closure (full “done” gate before Pi):**

```bash
./scripts/stack-closure-test.sh
# Optional one-shot Qdrant full sync:
STACK_FULL_QDRANT_SYNC=1 ./scripts/stack-closure-test.sh
```

## What the script checks

| Step | Validates |
|------|-----------|
| `verify-production.sh` | Prime :8000, VM200 embed :8081, rerank :8082, librarian :8083, daemon, health, vault |
| `cargo test -p gzmo-core --lib` | Unit tests (41 tests, 1 ignored live gateway) |
| Gateway JSON | Lenient parse + reasoning-trace extraction |
| Prime chat | Non-empty response with `reasoning_format: none` |
| `gzmo spark` | Full spark pipeline (hypothesis must not fail parse) |
| Daemon | Process running; warns on historical panic in log |

## After reboot

```bash
./scripts/start-production.sh --daemon
./scripts/p1-readiness-test.sh
```

## Known acceptable warnings

- **Sovereign :8010** down — FrankenMoE parked
- **Spark verification abstain** — `promoted: false` when citations don’t match vault spans (by design)
- **Historical daemon panic** in old logs — fixed in orchestrator UTF-8 truncation (restart daemon)

## Stack closure (2026-06-01)

| Item | Status |
|------|--------|
| VM200 embed `:8081`, rerank `:8082`, librarian `:8083` | Done |
| Daemon: dream → session_distill → Qdrant sync cron | Done |
| Spark off-peak cron (`03:30`, `22:30` UTC) | Done |
| Prime user systemd | `scripts/install-prime-systemd.sh` |
| Shared Neo4j MCP for Cursor | `scripts/install-shared-mcp.sh` |
| Pi Layer 2 (`pi-mcp-adapter`) | **Next** — after closure test passes |

## Optional

- Local embed `:8002` — fallback only if VM200 down
- `QdrantVault` in Rust — deferred; daily `sync-vault-to-qdrant.py` mirror is canonical
