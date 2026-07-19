# GZMO-next lab runbook — Workstation (not production)

**Production living host is CT101** (restored 2026-07-17). See
[CT101_BOUNDARY.md](CT101_BOUNDARY.md),
[CT101_RESTORE_LIVING.md](CT101_RESTORE_LIVING.md), and
[ADR-0003-one-instance-metabolism.md](ADR-0003-one-instance-metabolism.md).

This runbook covers the **workstation lab/dev stack** only:
`config/gzmo.toml` → `data-next/`, optional `gzmo serve` / `gzmo-scheduler`
for beat-gates. Operator frontend remains `gzmo` / `gzmo chat` on the
workstation; Prime `:8000` is CT101's local fallback.

**After 2026-07-17 restore:** `gzmo-serve` and `gzmo-scheduler` stay **disabled**
by default. Never enable overnight `gzmo serve` while CT101 `gzmo-daemon` is
the living writer.

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
export GZMO_CONFIG=$GZMO_CLONE_ROOT/GZMO/config/gzmo.toml
export LLM_URL=http://127.0.0.1:8000
export CARGO_TARGET_DIR=$GZMO_CLONE_ROOT/temp-bench/target
```

`[assembly] = "lab"` only affects transitional `gzmo daemon` / `gzmo assemble`.
**`gzmo serve` ignores assembly backends** and always runs typed Rust jobs.

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
**Soft-fail satellite:** OKForge wiki push at `[wiki] push_cron_*` (default
05:30 UTC) — records `latest-wiki.json` but does **not** affect metabolism GREEN.
No chaos / KG on this path. Writes `data-next/scheduler-runs/`.
systemd: `gzmo-serve.service` (+ `gzmo-serve.service.d/okforge.conf` for `OKFORGE_TOKEN`).

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

## Lab parity: `gzmo-scheduler` (offline by default)

Thin cron that spawns Little Tools Lab recipes for beat-gates only. **Do not**
enable alongside `gzmo-serve` for overnight — both write `scheduler-runs/`.

```bash
# Explicit beat-gate session only:
systemctl --user stop gzmo-serve.service
systemctl --user start gzmo-scheduler.service
# …run assemble / beat-gate…
systemctl --user stop gzmo-scheduler.service
systemctl --user start gzmo-serve.service
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

### `gzmo serve` overnight map (living)

| Loop | Schedule (UTC) | Runner |
|------|----------------|--------|
| Dream | `[dreams]` cron | typed `DreamEngine` |
| Distill | `[session_distill]` cron | typed `SessionDistillEngine` |
| Promote | `[metabolism]` promote | vault → honeypot |
| Embed | `[metabolism]` embed | backfill + Qdrant sync |
| Spark | `[spark]` cron_hours | typed `SparkEngine` |
| Wiki OKForge | `[wiki]` push_cron (05:30) | soft-fail `wiki_okf` → `latest-wiki.json` |

Lab recipe map below applies only when `gzmo-scheduler` is explicitly re-enabled for a beat-gate session.

### Scheduler loop → lab recipe map (offline / beat-gate only)

| Loop | Schedule (UTC) | Lab recipe |
|------|----------------|------------|
| Dream | 01:00 daily | `session-to-dream.sh --live` |
| Qdrant sync | 01:45 daily | `qdrant-vault-sync.sh` |
| Ingest batch | 02:00 daily | `ingest-smoke.sh --live` (watcher off; batch only) |
| Session distill | 02:15 daily | `synapse-distill-handoff.sh --live` |
| Spark | 03:30, 22:30 | `cognition-smoke.sh --live --vault data-next/vault.db --spark-run` |
| Ops health | startup | `ops-smoke.sh --live` |
| Config handoff | 04:00 daily | `gzmo-handoff.sh --live --apply --gzmo-config config/gzmo-next-fused.toml` |
| KG reconcile | 04:30 daily | `kg-reconcile-smoke.sh --live` (`[kg_reconcile] dry_run` until verified) |
| Recall floor | Sunday 05:15 | `recall-eval-weekly.sh` → `data-next/recall-report.json` |
| Pedagogy | Sunday 06:00 | `pedagogy-smoke.sh --live` ([ADR-0002](../../little-tools-lab/docs/adr/0002-pedagogy-chaos-scheduler-lab-only.md)) |
| Cabinet feed | Sunday 06:30 | `cabinet-feed.sh --live` (one-shot; not PulseLoop) |
| Discovery (optional) | **not armed by default** | `discovery-smoke.sh --live` via `beat-gate --loop discovery` or a host weekly cron. Do **not** add a DiscoveryEngine to `gzmo-scheduler`. Arm only after fixture beat-gate stays green. |

**Wiki plane:** OKForge OKCP → `gzmo/gzmo-next-memory` via `gzmo serve` satellite (or `gzmo wiki push`). Requires `OKFORGE_TOKEN` (`~/.config/okforge/env`, loaded by `gzmo-serve.service.d/okforge.conf`) and local `okforge.service` on `:3000`.

**Observatory:** in-forge at `http://127.0.0.1:3000/observatory` (`okforge.service`). The FastAPI sidecar `:7777` / `gzmo-observatory.service` is retired.

**Production gate:** [`docs/OKFORGE_PRODUCTION.md`](OKFORGE_PRODUCTION.md) + `bash ~/Schreibtisch/okforge/scripts/production-smoke.sh`

Job results land in `data-next/scheduler-runs/{job}-{timestamp}.json` (plus
`latest.json` and `wiki-push-latest.json`).

**Missed-run watchdog (soft-fail):** `gzmo status` / `gzmo serve` poll write
`scheduler-runs/latest-watchdog.json`. If `latest-distill` or `latest-dream` is
missing or older than 26h (override: `GZMO_METABOLISM_STALE_SECS`), verdict shows
**YELLOW — metabolism stale** without flipping core GREEN job math to RED.

**Nightburst (compressed proof / Arena):** when the machine cannot stay up for
calendar nights, use burst cycles instead:

```bash
# Seed distinctive session JSON under data-next/sessions/, then:
gzmo distill <session-id> && gzmo memory promote && gzmo memory embed && gzmo dream
gzmo memory search '<seeded fact>'
# Arena + sanitized scoreboard (local HTML; OKForge /observatory stays agent-discovery):
bash scripts/arena-night.sh
bash scripts/nightburst-scoreboard.sh
# open data-next/arena/scoreboard.html
systemctl --user stop gzmo-serve   # free the machine when the sitting ends
```

See [`STACK_OPPORTUNITY_MAP.md`](STACK_OPPORTUNITY_MAP.md) and `data-next/recall-proof.md`.

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
gzmo session close --takeaway "…"       # append durable takeaways → distill queue (`--now` runs distill)
gzmo dream compact [--max-chars N] [--archive-sessions-days 30] [--dry-run]
bash scripts/serendipity-digest.sh      # spark → data-next/serendipity/digest-YYYY-MM-DD.md
bash scripts/faithfulness-ci.sh         # claims vs vault (FAITHFULNESS_MODE=fixture for offline)
bash scripts/organ-trace.sh             # living tool zoo from scheduler-runs
bash scripts/concept-review-gate.sh     # HOLD wiki concepts lacking vault evidence
bash scripts/wiki-push-gated.sh […]     # gate then `gzmo wiki push --require-gate`
bash scripts/nightburst-bundle.sh       # organ + faithfulness + concept-gate + digest + scoreboard
bash scripts/nightburst-scoreboard.sh   # refresh local scoreboard HTML
bash scripts/herdr-metabolism-link.sh   # link herdr plugin gzmo.metabolism (MCP + close ritual)
#   herdr plugin action invoke gzmo.metabolism.ensure-mcp
#   takeaway via overlay close-ritual, selection, or $(herdr plugin config-dir gzmo.metabolism)/takeaway.txt
bash scripts/hsp-metabolism-sonify.sh [--play]  # metabolism artifacts → MIDI/WAV motif
bash scripts/euro-night-aggregate.sh    # Arena history + metabolism €/night
bash scripts/price-window-suggest.sh    # Awattar ±2h distill/dream suggestion (no cron mutate)
bash scripts/price-shift-soft.sh        # soft shift note → scheduler-runs/latest-price-shift.json
#   GZMO_PRICE_SHIFT=1 on serve → delay distill/dream until suggested UTC
bash scripts/aos-status-feed.sh [--serve]  # AOS TelemetryPayload JSON (:8765 optional)
bash scripts/aos-gzmo-poll.sh [--check-http]  # refresh + verify AOS file/HTTP poll
bash scripts/concept-gate-webhook.sh [--serve]  # gate merge advice (:8766 POST /gate)
bash scripts/obolus-forge-mutate.sh     # pin Arena winners / sibling mutation proposals
bash scripts/ipw-route.sh [--task chat] # Intelligence-per-Watt route advice
bash scripts/cognition-pack.sh [--smoke]  # portable distill→recall contract + status
bash scripts/tinyfolder-drop.sh --demo  # drop markdown into data-next/inbox
bash scripts/beat-gate-kit.sh           # fixture→meta→gate organ promotion kit
bash scripts/zpd-tutor-lab.sh [--topic T]  # soft-fail ZPD tutor lab (not on GREEN)
bash scripts/okcp-marketplace.sh [--intent write]  # concept bundle export + gated write intent
bash scripts/pi-operator-glass.sh       # Pi-facing status glass (CLI remains canonical)
bash scripts/rapl-probe.sh              # why Arena is estimate vs RAPL (root-only energy_uj)
bash scripts/aos-ce-pin.sh              # golden-path SHA pin for AOS Customer Edition
bash scripts/escape-loop-kit.sh         # soft escape-loop dry-run (research; not on GREEN)
bash scripts/portable-core-inventory.sh # living vs gzmo-core-clean; default hold_rewrite
bash scripts/cognis-dialect-stub.sh     # weekend dialect over plan-gate (not production brain)
bash scripts/edge-fleet-sketch.sh       # hub/edge topology sketch (no sync)
bash scripts/product-stranger-path.sh   # stranger MCP install checklist (laptop product)
bash scripts/mcp-attach-check.sh        # Cursor/Pi mcp.json → ~/.gzmo (MCP_ATTACH_FIX=1 to rewire)
bash scripts/ct101-living-probe.sh      # soft CT101 smoke + dual-writer check (Keep)
bash scripts/ct101-takeaway-recall.sh   # living takeaway → distill → recall HIT
bash scripts/faithfulness-living.sh     # CORE_INSIGHT claims vs CT101 vault
bash scripts/takeaway-ritual-lab.sh     # session close --takeaway → distill enqueue (no --now)
bash scripts/dream-compact-lab.sh       # dream compact --dry-run (Keep plumbing; not on GREEN)
bash scripts/spine-demo.sh              # Keep pillars: product MCP + recall-proof (see SPINE_FOCUS.md)
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

## CT101 cutover — superseded (2026-07-17)

**Living production is CT101 again.** Do not migrate `data-next/` onto CT101
or freeze CT101 for a “next” promotion unless a new ADR says so. Historical
notes below are lab-only.

Per [CT101_BOUNDARY.md](CT101_BOUNDARY.md) there is no incremental grafting —
cutover would be one migration if GZMO-next were ever re-promoted. **Fresh `data-next/` remains
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
