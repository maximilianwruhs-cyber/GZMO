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

## Final Cloud + Sidecar Migration (2026-07-02)

**Executed by:** Antigravity (Google DeepMind)
**Target:** LXC101 Sidecar Homing

### Changes & Configuration
- Wrote deployment script `scripts/lxc101/deploy-gzmo-daemon.sh` and registered `gzmo-daemon.service` as a system service on LXC101 running under the newly created `maximilian` user.
- Updated `gzmo.toml` to:
  - Enable cloud engine (`active_mode = "cloud"`, `default_engine = "cloud"`, `cloud_first_background = true`).
  - Route all cognition mappings (`dream_*`, `spark_*`, `ingest_*`, `distill_*`) to OpenRouter (`deepseek/deepseek-v4-flash`).
  - Disable Obolus energy sampler (`energy_sampler_enabled = false`) since raw hardware counters do not exist inside LXC101.
  - Rewrote absolute paths (`/home/maximilian-wruhs` -> `/home/maximilian` and `/opt/gzmo`) and redirected Neo4j, Redis, and Qdrant endpoints to `localhost`.
- Configured EnvironmentFile `/opt/gzmo/.env` containing the `GZMO_OPENROUTER_KEY`.
- Migrated all stateful data (`vault.db`, `data/`, `memory/`, `skills/`, `wiki/`, and sibling `gzmo_skills/` folder) to LXC101.
- Created relative symlinks inside `/opt/gzmo/survey_GZMO/` pointing to `/opt/gzmo/` directories so that verification and evaluation scripts run identically to the workstation environment.

### Validation Results
- Stopped workstation `gzmo-prime.service` and `gzmo-daemon.service` (ready for workstation wipe).
- Running on sidecar (LXC101):
  - `verify-production.sh` -> **PASS** (Prime check skipped in cloud mode)
  - `sovereignty-verify.sh` -> **PASS** (1 warning: active_mode=cloud)
  - `eval-quick.sh` -> **PASS** (offline contract and retrieval probes)
  - Manual `gzmo spark` -> **PASS** (Hypothesis successfully generated using OpenRouter, links quarantined, and relations committed to Neo4j graph).
