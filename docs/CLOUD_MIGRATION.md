# Cloud + sidecar migration log

**Executed:** 2026-06-24

## Operator intent

Move background cognition to cloud LLM while keeping sidecar persistence (LXC101) and retrieval (VM200). Retire local Prime to free workstation VRAM after validation.

## Config (`gzmo.toml`)

- `compliance.allow_cloud_engine = true`
- `routing.cloud_first_background = true`
- `engine.active_mode = cloud`

## Infrastructure state

| Component | Status |
|-----------|--------|
| Prime `:8000` | Stopped + disabled (`gzmo-prime.service`) |
| Cloud LLM | OpenRouter `deepseek/deepseek-v4-flash` + Gemini fallback |
| VM200 `:8081` | Unchanged (embed + rerank) |
| LXC101 | Unchanged (Neo4j, Qdrant, Redis) |
| gzmo daemon | Workstation, cloud mode |

## Validation

- `sovereignty-verify.sh` — PASS (warn: active_mode=cloud)
- `eval-quick.sh` tier 0 — PASS
- `live_cloud_probe` — PASS
- `verify-production.sh` — PASS (Prime skipped in cloud mode)

## Code changes (health probes)

- `gzmo-core/src/health.rs` — probe `active_engine()` not hardcoded Prime
- `gzmo-cli/src/health_cmd.rs` — same
- `scripts/start-production.sh` — skip Prime when cloud mode
- `scripts/verify-production.sh` — skip Prime check in cloud mode
- `scripts/systemd/gzmo-daemon.service` — removed `After=gzmo-prime.service`

## Rollback

```toml
active_mode = "local"
cloud_first_background = false
allow_cloud_engine = false
```

```bash
systemctl --user enable --now gzmo-prime.service
systemctl --user restart gzmo-daemon.service
```

## Rollback to local (2026-06-25)

Operator decision: return to **local-first** cognition on Prime (`:8000`).

- `active_mode = "local"`
- `cloud_first_background = false`
- `allow_cloud_engine = false`
- `gzmo-prime.service` re-enabled

Cloud path remains documented for future retry; not active in steady state.

Local Prime baseline for dream/spark comparison: [`M3_LOCAL_BASELINE.md`](./M3_LOCAL_BASELINE.md).

## Deferred

- Phase 3: daemon to LXC101
- Phase 4: SQLite vault → Qdrant primary
- Obolus USD budget caps
- Pi completions routing to cloud
