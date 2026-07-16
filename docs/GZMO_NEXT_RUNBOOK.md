# GZMO-next Runbook — Workstation Instance (Lab Backend)

GZMO-next is the second, standalone GZMO instance on the workstation. Its loops
are backed by Little Tools Lab recipes (bash + file artifacts) instead of the
inline engines. The long-running process is the thin `gzmo-scheduler` binary
(see below); the `gzmo` binary remains the operator frontend. CT101 legacy is
untouched — see [CT101_BOUNDARY.md](CT101_BOUNDARY.md).

Verified on workstation 2026-07-10: S1 live green, S2 beat-gate green for all
four loops (config, ops, cognition, knowledge), daemon lab dispatch green.

## Instance layout

| Asset | Legacy (CT101 semantics) | GZMO-next |
|-------|--------------------------|-----------|
| Config | `GZMO/gzmo.toml` | `GZMO/config/gzmo-next.toml` |
| Data root | `GZMO/data/` | `GZMO/data-next/` |
| Vault | `data/vault.db` | `data-next/vault.db` |
| Sessions | `data/sessions/` | `data-next/sessions/` |
| Distill queue | `gzmo:distill:pending` (Redis) | Redis `gzmo-next:distill:pending` + file fallback `data-next/distill-queue/` |
| Dreams | `DREAMS.md` | `data-next/DREAMS.md` |
| Fused calibration | applied to config | written to `config/gzmo-next-fused.toml` (review, never clobbers live config) |

## Env contract

Every operator command and the daemon itself run with:

```bash
export GZMO_CLONE_ROOT=/home/gzmo/github-clone
export GZMO_INSTANCE=next
export GZMO_CONFIG=$GZMO_CLONE_ROOT/GZMO/config/gzmo-next.toml
export LLM_URL=http://127.0.0.1:8000          # lab recipes read this
export CARGO_TARGET_DIR=$GZMO_CLONE_ROOT/temp-bench/target  # shared lab binaries
```

Guardrail: `[assembly] = "lab"` backends only activate when `GZMO_INSTANCE=next`.
Any other value (or unset) forces every loop to Inline, so the same binary and
config parser stay CT101-safe. The daemon logs the resolved backends at boot:

```
Assembly backends resolved instance=next distill="lab" dream="lab" spark="lab" ops_health="lab" config_handoff="lab"
```

## Required services

Post-cutover (2026-07-15) the memory plane is **enabled** in
[`config/gzmo-next.toml`](../config/gzmo-next.toml). See
[CT101_BOUNDARY.md](CT101_BOUNDARY.md) for the cutover checklist. Ingest
watcher stays off until promotion quality is gated.

| Service | Endpoint | Required? |
|---------|----------|-----------|
| Prime LLM (llama.cpp) | `http://127.0.0.1:8000/v1` | Yes — spark run, verify-suite, calibration, librarian |
| Librarian (session extract) | `http://127.0.0.1:8000/v1` (Prime) | Preferred — distill falls back to heuristic if unreachable |
| Embeddings | `http://192.168.31.110:8081/v1` (VM200) | Yes — `[embeddings] enabled = true` |
| Rerank | `http://192.168.31.110:8081/v1` (VM200) | Yes — `[rerank] enabled = true` |
| Qdrant | `http://127.0.0.1:6333` | Yes — `[qdrant] enabled = true`, collection `honeypot` |
| Redis | `redis://127.0.0.1:6379` | Yes — `[redis] enabled = true`, queue `gzmo-next:distill:pending` |
| Neo4j (MCP memory) | `bolt://127.0.0.1:7687` | Yes — `[[mcp_servers]]` memory in toml |
| Ingest watcher | — | No — `[ingest] enabled = false` (workstation v1) |

## Canonical long-running process: `gzmo serve` (ADR-0003)

The living overnight runner is **`gzmo serve`**: typed Rust jobs
(distill → promote → embed → dream/spark) + distill BRPOP worker.
No chaos, no wiki/KG/discovery on this path. Writes
`data-next/scheduler-runs/`. systemd: `gzmo-serve.service`.

```bash
export GZMO_INSTANCE=next
export GZMO_CONFIG=$GZMO_CLONE_ROOT/GZMO/config/gzmo.toml
cd $GZMO_CLONE_ROOT/GZMO
cargo build --release -p gzmo-cli
./target/release/gzmo serve

# Or: systemctl --user enable --now gzmo-serve.service
# Stop: kill "$(cat /tmp/gzmo-serve.pid)"
```

Operator proof: `gzmo status` → **Overnight metabolism** section.

## Optional lab parity: `gzmo-scheduler`

Thin cron that spawns Little Tools Lab recipes (beat-gate / parity). Not the
metabolism authority. Requires `GZMO_INSTANCE=next` and lab assembly backends.

```bash
cargo build --release -p gzmo-scheduler
./target/release/gzmo-scheduler   # lock: /tmp/gzmo-scheduler.pid
```

## Transitional: `gzmo daemon` with lab branches

The legacy daemon also honors `[assembly] = "lab"` under `GZMO_INSTANCE=next`,
but it still boots the full inline stack (engines, MCP, chaos) before
branching. Kept for reference/fallback; prefer `gzmo serve`.

```bash
# Start (foreground; singleton lock at /tmp/gzmo_rust.pid)
cd $GZMO_CLONE_ROOT/GZMO
gzmo daemon

# Stop
kill "$(cat /tmp/gzmo_rust.pid)"

# If the daemon refuses to start after a crash, reclaim the stale lock:
rm -f /tmp/gzmo_rust.pid
```

Note: `/tmp/gzmo_rust.pid` is shared per host — do not run legacy and next
`gzmo daemon` simultaneously. `gzmo-scheduler` uses its own lock and can
coexist with a legacy daemon during transition (but not with a next-instance
daemon, or jobs would double-fire).

### Scheduler loop → lab recipe map

| Loop | Schedule (UTC) | Lab recipe |
|------|----------------|------------|
| Dream | 01:00 daily | `session-to-dream.sh --live` → hooks `wiki-okforge-push.sh` |
| Qdrant sync | 01:45 daily | `qdrant-vault-sync.sh` |
| Ingest batch | 02:00 daily | `ingest-smoke.sh --live` (watcher off; batch only) |
| Session distill | 02:15 daily | `synapse-distill-handoff.sh --live` → hooks wiki OKForge push |
| Spark | 03:30, 22:30 | `cognition-smoke.sh --live --vault data-next/vault.db --spark-run` |
| Ops health | startup | `ops-smoke.sh --live` |
| Config handoff | 04:00 daily | `gzmo-handoff.sh --live --apply --gzmo-config config/gzmo-next-fused.toml` |
| KG reconcile | 04:30 daily | `kg-reconcile-smoke.sh --live` (`[kg_reconcile] dry_run` until verified) |
| Recall floor | Sunday 05:15 | `recall-eval-weekly.sh` → `data-next/recall-report.json` |
| Wiki OKForge catch-up | 05:30 daily | `wiki-okforge-push.sh --live` (if recipe hooks missed) |
| Pedagogy | Sunday 06:00 | `pedagogy-smoke.sh --live` ([ADR-0002](../../little-tools-lab/docs/adr/0002-pedagogy-chaos-scheduler-lab-only.md)) |
| Cabinet feed | Sunday 06:30 | `cabinet-feed.sh --live` (one-shot; not PulseLoop) |
| Discovery (optional) | **not armed by default** | `discovery-smoke.sh --live` via `beat-gate --loop discovery` or a host weekly cron. Do **not** add a DiscoveryEngine to `gzmo-scheduler`. Arm only after fixture beat-gate stays green. |

**Wiki plane:** OKForge OKCP → `gzmo/gzmo-next-memory` (`gzmo wiki push`). Requires `OKFORGE_TOKEN` (`~/.config/okforge/env`, loaded by `gzmo-scheduler` drop-in) and local `okforge.service` on `:3000`. Not CT101 local WikiEngine sync.

**Observatory:** in-forge at `http://127.0.0.1:3000/observatory` (`okforge.service`). The FastAPI sidecar `:7777` / `gzmo-observatory.service` is retired.

**Production gate:** [`docs/OKFORGE_PRODUCTION.md`](OKFORGE_PRODUCTION.md) + `bash ~/Schreibtisch/okforge/scripts/production-smoke.sh`

Job results land in `data-next/scheduler-runs/{job}-{timestamp}.json` (plus
`latest.json` and `wiki-push-latest.json`) for the Observatory Body panel.

## Operator commands

```bash
gzmo instance status                     # instance, paths, skills_root, effective assembly backends
gzmo config promote-fused --diff         # review sibling gzmo-next-fused.toml vs live
gzmo config promote-fused --diff --apply # merge calibration into live (never full-clobber next)
gzmo assemble ops --live                 # health chain (+ sidecar/queue metrics)
gzmo status                              # deterministic ecosystem snapshot (paths + probes)
gzmo health                              # strict subsystem probes
gzmo assemble cognition --live          # distill → gate → spark → recall (instance vault)
gzmo assemble handoff --live --apply    # bench → fuse → gzmo-next-fused.toml on gate pass
gzmo distill                            # distill data-next/sessions/ into the vault
gzmo chat                               # sessions persist to data-next/sessions/
# Skills (next): authoritative root = GZMO/skills/ (see gzmo instance status → skills_root).
# gzmo_skills/ remains CT101/bridge auxiliary only — see gzmo_skills/BRIDGE.md.
```

## Sidecars (Docker only)

```bash
cd ~/database-cluster && docker compose up -d
sudo systemctl status gzmo-sidecars
```

Do **not** enable `gzmo-sidecar-{qdrant,redis}.service` user units (legacy native
binaries; disabled on purpose).
## Vault seeding (fresh instance)

A brand-new instance has an empty vault, which makes spark/cognition live paths
skip. Seed it by distilling at least one session:

```bash
cp $GZMO_CLONE_ROOT/session-distill/fixtures/session.json \
   $GZMO_CLONE_ROOT/GZMO/data-next/sessions/seed-session.json
# ensure each message has "is_meta": false (gzmo-core Session schema)
gzmo distill
```

Spark anchors need `promoted_at` at least `[spark].anchor_min_stale_days` old;
freshly seeded facts appear as recents immediately and become anchors as they age.

## Smoke + beat-gate checklist (S2)

Run before trusting a rebuilt or migrated instance:

```bash
cd $GZMO_CLONE_ROOT/little-tools-lab
export VAULT_PATH=$GZMO_CLONE_ROOT/GZMO/data-next/vault.db

bash scripts/live-smoke-all.sh                                              # S1
bash scripts/beat-gate.sh --loop config    --live --meta /tmp/beat-config.json
bash scripts/beat-gate.sh --loop ops       --live --meta /tmp/beat-ops.json
bash scripts/beat-gate.sh --loop cognition --live --meta /tmp/beat-cognition.json
bash scripts/beat-gate.sh --loop knowledge --live --meta /tmp/beat-knowledge.json
python3 scripts/validate-schemas.py
```

All four must print `PASS: lab beats incumbent`. Metas conform to
[`schemas/beat-meta.json`](../../little-tools-lab/schemas/beat-meta.json).

## CT101 cutover (future, single migration)

Per [CT101_BOUNDARY.md](CT101_BOUNDARY.md) there is no incremental grafting —
cutover is one migration when GZMO-next is proven. **Fresh `data-next/` remains
valid** without import (stretch S3 decision gate).

Tooling (stretch S3):

```bash
# Read-only compare (next vs CT101 snapshot / local copy)
python3 scripts/vault-diff.py \
  --left data-next/vault.db \
  --right /path/to/ct101-vault.db

# Print freeze/backup/copy/sync checklist; refuses if scheduler PID lock live
bash scripts/vault-migrate.sh --dry-run \
  --src /opt/gzmo/data/vault.db \
  --dest data-next/vault.db

# Explicit apply only after operator decision (creates .bak-* first)
# bash scripts/vault-migrate.sh --apply --yes --src … --dest …
```

Manual cutover outline:

1. Freeze CT101 (`systemctl stop` the legacy daemon; snapshot the container).
2. Copy the clone tree (GZMO + little-tools-lab + piece repos) to the new host;
   the env contract above is the only host-specific configuration.
3. Migrate memory once via `vault-migrate.sh --apply --yes` (or copy `vault.db` →
   `data-next/vault.db`), replay any pending distill queue, re-run `gzmo distill`.
4. Run the full S2 checklist above on the new host — all green before DNS/cron
   ownership moves.
5. Point operators at the new instance; CT101 stays frozen as reference until
   decommissioned. No flag flips on CT101, ever.

## Calibration cadence

| Cadence | Action | Target |
|---------|--------|--------|
| **Daily 04:00 UTC** | `gzmo-handoff.sh --live --apply` (scheduler) | Sibling `config/gzmo-next-fused.toml` only — **HOLD** if benchmark gate fails |
| **Weekly / monthly** | Human review | `gzmo config promote-fused --diff` then `--apply` (section merge into live) |
| **Never** | Scheduler auto-merge into live `gzmo-next.toml` | — |

Artifacts: `data-next/handoff/last-gzmo-handoff-meta.json`, `last-fuse-meta.json`, `last-handoff-gate.json` (`gate_passed`, `verify_pass_rate`).

## Weekly graph triage

After Sunday dream / when Observatory **Graph drift** shows alerts:

1. Read `anomaly_count` from `data-next/dream-stats.json` (also on `gzmo status` → Graph drift).
2. Open `data-next/graph-ledger.jsonl` (tail recent lines) and note which sessions/entities drifted.
3. Decide action: ignore (noise), re-run dream with `--live`, or file a lab issue — do **not** auto-merge fused config as a “fix”.

Sunday 05:15 UTC the scheduler also refreshes `data-next/recall-report.json` (recall floor).

## Weekly mentor hour (ADR-0002 — weekly cron + manual)

Pedagogy runs **Sunday 06:00 UTC** via `gzmo-scheduler` after `beat-gate --loop pedagogy` is green.
Manual rehearsal still works:

```bash
export GZMO_INSTANCE=next
export GZMO_CONFIG=$GZMO_CLONE_ROOT/GZMO/config/gzmo-next.toml
gzmo assemble pedagogy --fixture   # rehearsal
gzmo assemble pedagogy --live      # same recipe the cron uses
```

Cabinet crystallize runs **Sunday 06:30 UTC** via `cabinet-feed.sh` (one-shot). PulseLoop,
dice-scheduler, adaptive-tempo, and research-budget stay **off** the thin scheduler
([ADR-0002](../../little-tools-lab/docs/adr/0002-pedagogy-chaos-scheduler-lab-only.md)).

### Chat rituals (not cron)

| Ritual | Piece | How |
|--------|-------|-----|
| Research budget | `research-budget check/spend` | Gate autonomous research tokens in chat |
| Calibrate theatre | `/calibrate` → `bench-to-fuse --fixture` | Experience C rehearsal |
| PulseLoop / `/chaos` | `gzmo chat` | Continuous Lorenz — never `gzmo-scheduler` |
