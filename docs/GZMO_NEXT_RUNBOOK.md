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
| Distill queue | `gzmo:distill:pending` (Redis) | file queue `data-next/distill-queue/` |
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

| Service | Endpoint | Required? |
|---------|----------|-----------|
| Prime LLM (llama.cpp) | `http://127.0.0.1:8000/v1` | Yes — spark run, verify-suite, calibration |
| Librarian | `http://127.0.0.1:8083/v1` | No — distill falls back to heuristic |
| Embeddings / Qdrant / Redis / Neo4j | — | No — disabled in `gzmo-next.toml` for v1 |

## Canonical long-running process: `gzmo-scheduler`

The recommended way to run GZMO-next is the dedicated **`gzmo-scheduler`**
binary (`GZMO/gzmo-scheduler/`): a thin cron runner that only spawns the lab
recipes below. It links none of the inline engines — no DreamEngine, no
SparkEngine, no MCP, no chaos — so it compiles in seconds and cannot regress
into the monolith. At startup it refuses to run unless `GZMO_INSTANCE=next`
and every `[assembly]` loop is `"lab"`.

```bash
# Start (foreground; singleton lock at /tmp/gzmo-scheduler.pid)
cd $GZMO_CLONE_ROOT/GZMO
cargo build --release -p gzmo-scheduler
./target/release/gzmo-scheduler

# Stop
kill "$(cat /tmp/gzmo-scheduler.pid)"

# Reclaim a stale lock after a crash:
rm -f /tmp/gzmo-scheduler.pid
```

It reads the same `GZMO_CONFIG` (`gzmo-next.toml`) and runs the same loop →
recipe map as the transitional path below.

## Transitional: `gzmo daemon` with lab branches

The legacy daemon also honors `[assembly] = "lab"` under `GZMO_INSTANCE=next`,
but it still boots the full inline stack (engines, MCP, chaos) before
branching. Kept for reference/fallback; prefer `gzmo-scheduler`.

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
| Session distill | 02:15 daily | `synapse-distill-handoff.sh --live` |
| Dream | 01:00 daily | `session-to-dream.sh --live --output data-next/DREAMS.md` |
| Spark | 03:30, 22:30 | `cognition-smoke.sh --live --vault data-next/vault.db --spark-run` |
| Ops health | startup | `ops-smoke.sh --live` |
| Config handoff | 04:00 daily | `gzmo-handoff.sh --live --apply --gzmo-config config/gzmo-next-fused.toml` |

## Operator commands

```bash
gzmo assemble ops --live                 # health chain
gzmo assemble cognition --live          # distill → gate → spark → recall (instance vault)
gzmo assemble handoff --live --apply    # bench → fuse → gzmo-next-fused.toml on gate pass
gzmo distill                            # distill data-next/sessions/ into the vault
gzmo chat                               # sessions persist to data-next/sessions/
```

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
cutover is one migration when GZMO-next is proven:

1. Freeze CT101 (`systemctl stop` the legacy daemon; snapshot the container).
2. Copy the clone tree (GZMO + little-tools-lab + piece repos) to the new host;
   the env contract above is the only host-specific configuration.
3. Migrate memory once: copy `vault.db` → `data-next/vault.db`, replay any
   pending distill queue, re-run `gzmo distill`.
4. Run the full S2 checklist above on the new host — all green before DNS/cron
   ownership moves.
5. Point operators at the new instance; CT101 stays frozen as reference until
   decommissioned. No flag flips on CT101, ever.
