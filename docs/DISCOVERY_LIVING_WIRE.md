# Discovery living wire (CT101)

**Purpose:** Keep Pi discovery attached to the living stack and **probe-first**.  
**Related:** [CT101_DEPLOY.md](./CT101_DEPLOY.md), [CT101_RESTORE_LIVING.md](./CT101_RESTORE_LIVING.md)

## What’s living

| Piece | Path / unit |
|-------|-------------|
| Vault + daemon | `/opt/gzmo/` + `gzmo-daemon.service` |
| Binary | `/opt/gzmo/current/target/release/gzmo` |
| Config | `/opt/gzmo/gzmo.toml` |
| Discovery scripts | `/home/maximilian/gzmo_skills/` |
| Timer | `pi-mentor-discovery.timer` (user maximilian) |
| Pi agent home | `/home/maximilian/.pi/agent/` |

## Pi / model cost split (must stay true)

| Surface | Model | Why |
|---------|--------|-----|
| **Discovery cycles** | `DISCOVERY_PI_MODEL` in `config.env` (default `google/gemini-2.5-flash-lite`) | Cheap; probe-first gate enforces tools |
| **Interactive Pi** | `gemini-2.5-flash` in `~/.pi/agent/settings.json` | Cheap tool use; not GLM |
| **Daemon cognition** | `z-ai/glm-5.2` with `reasoning_effort = "medium"` in `/opt/gzmo/gzmo.toml` | Overnight quality without `xhigh` credit burn |

`~/.pi/agent/mcp.json` (living attach — independent of model):

- `gzmo-memory` → `/opt/gzmo/current/target/release/gzmo mcp-serve`
- `GZMO_CONFIG=/opt/gzmo/gzmo.toml`
- Package: `npm:pi-mcp-adapter`

Do **not** set interactive Pi default to GLM 5.2 — discovery already overrides via `--model`; GLM on settings only burns credits in manual Pi sessions.

Systemd discovery service:

- `GZMO_ROOT=/opt/gzmo/current` (not `survey_GZMO` as the mental SoT)
- `TimeoutStartSec=45min` (long cycles must not be killed mid-flight)

## Lock / timer hygiene

| Knob | Value | Why |
|------|--------|-----|
| `OnUnitInactiveSec` | **600s** (`DISCOVERY_TIMER_INTERVAL_SEC`) | Was 120s → `lock_skip` storms while a cycle held `.cycle.lock` |
| `DISCOVERY_LOCK_WAIT_SEC` | **1800** | Busy lock → `flock -w` wait + `lock_wait` metric, not immediate exit |
| After wait | **Continue this cycle** (do not exit 0 and defer another 600s) |
| Lock file | `$PI_MENTOR_DISCOVERY_DATA/.cycle.lock` | Single-flight per host |

Healthy signature in `cycle-metrics.jsonl`: occasional `event:"lock_wait"` then a normal cycle with `bash_calls >= 1`. Frequent `lock_skip` means the old immediate-exit path is still deployed.

## Thin MCP ops tools

Living `gzmo mcp-serve` also exposes (read-only):

| Tool | Returns |
|------|---------|
| `gzmo_ops_health` | Same probes as `gzmo health` (LLM, Qdrant, honeypot drift, Redis, Neo4j) |
| `gzmo_discovery_status` | `state.json` + last `cycle-metrics` line (`bash_calls`, `probe_required_failed`, lock presence) |

Discovery data dir: `PI_MENTOR_DISCOVERY_DATA` / `DISCOVERY_DATA_DIR` / default `/home/maximilian/gzmo_skills/data/pi-mentor-discovery`.

Pi habitual ops glance (interactive): `system.md` orders `gzmo_memory_status` → `gzmo_ops_health` → `gzmo_discovery_status`.

## Probe-first gate

In `pi-mentor-discovery-cycle.sh`:

- After dialogue, count `bash|gzmo_health|gzmo_ops_health|systemctl|after-boot-verify` in the session log
- If `bash_calls < 1` → log `PROBE_REQUIRED`, set `probe_required_failed=true`, **do not publish** / do not update `latest.md`

Opener / interactive Pi: first probe may be `bash`, `gzmo_health`, or MCP `gzmo_ops_health`. Habitual ops glance is `gzmo_memory_status` → `gzmo_ops_health` → `gzmo_discovery_status` (see `~/.pi/agent/system.md`).

## KPI

```bash
ssh ct101 'tail -50 /home/maximilian/gzmo_skills/data/pi-mentor-discovery/logs/cycle-metrics.jsonl' \
  | python3 -c '
import sys,json
n=b=0
for line in sys.stdin:
  o=json.loads(line)
  if "bash_calls" not in o: continue
  n+=1
  if o["bash_calls"]>0: b+=1
print(f"cycles={n} with_bash={b} rate={b/n if n else 0:.0%}")
'
```

Target: high `with_bash` rate. Zero-bash publishes should be impossible.

## One manual cycle

```bash
ssh ct101
sudo -u maximilian XDG_RUNTIME_DIR=/run/user/$(id -u maximilian) bash -lc '
  cd ~/gzmo_skills
  systemctl --user stop pi-mentor-discovery.timer
  bash scripts/stop-pi-mentor-discovery-session.sh || true
  DISCOVERY_SESSION_SIZE=small bash scripts/start-pi-mentor-discovery-session.sh small
  # timer may arm; or:
  # systemctl --user start pi-mentor-discovery.service
  # watch logs:
  # tail -f data/pi-mentor-discovery/logs/cycle.log
'
```

Success: latest `cycle-metrics.jsonl` line has `bash_calls >= 1` and `probe_required_failed: false` (or absent/`0`).

## Product gate (daemon/vault)

```bash
bash scripts/ct101-living-smoke.sh
```

## Proven (2026-07-17 Step 1)

- Pi rewired to living GLM 5.2 + `/opt/gzmo/gzmo.toml`
- Cycle `2026-07-17T13-04-13Z` pillar C: **`bash_calls=2`**, `probe_required_failed=0`, published

## Deep dig smoking guns (2026-07-17)

| Finding | Severity | Status |
|---------|----------|--------|
| Skills cwd `gzmo_skills/gzmo.toml` hijacked selfheal/`gzmo health` → localhost:8000, honeypot=1224, false DEGRADED + failed 13:09 cycle | **P0** | Fixed: force `GZMO_CONFIG`, quarantine lab toml, living vault path for HEAL-6 |
| `gzmo-root.sh` rejected `/opt/gzmo/current` as “polluted” | P1 | Fixed: accept `current` \|\| `survey_GZMO` |
| Mentor socket file present but **not listening** — living source has **no mentor module**; teach falls back to OpenRouter | **P0 architecture** | **Restored 2026-07-17** — chaos-free Unix mentor on dedicated thread; `ping`/`status`/`teach` live at `/opt/gzmo/data/gzmo_mentor.sock` |
| Honeypot↔Qdrant drift 37976 vs 24603 (65%) | P1 | Still open (sync upserted ~24k) |
| Selfheal HEAL-3 probed Prime via localhost:8000 tunnels | P2 | **Fixed:** probe `/opt/gzmo/gzmo.toml` `[engine.local]` / `.184`; skip obsolete HEAL-1 tunnels on living LAN |

## Living mentor (restored 2026-07-17)

Chaos-free Unix mentor on the daemon (dedicated OS thread — CT101 often has 1 tokio worker; dream/spark must not starve accept):

| Item | Value |
|------|--------|
| Socket | `/opt/gzmo/data/gzmo_mentor.sock` |
| Methods | NDJSON `ping` / `status` / `reload` / `teach` |
| CLI | `GZMO_CONFIG=/opt/gzmo/gzmo.toml gzmo mentor ping\|status\|teach "…"` |

When the socket answers, Pi `gzmo_mentor_teach` must not tag `fallback:"openrouter"`.

Discovery preflight now **requires** a living socket `ping` (not merely “daemon process OR OpenRouter key”). Escape hatch: `DISCOVERY_ALLOW_MENTOR_FALLBACK=1`.

## New living baseline — done vs ground left

**In the living product (CT101) today**

| Slice | State |
|-------|--------|
| One living instance = CT101; workstation = operator + Prime | Done — [CT101_RESTORE_LIVING.md](./CT101_RESTORE_LIVING.md) |
| Probe-first discovery gate (`bash_calls` / `PROBE_REQUIRED`) | Done |
| Lock wait + 600s timer (no skip storms) | Done |
| Thin MCP: `gzmo_ops_health`, `gzmo_discovery_status` | Done |
| Config hijack (skills cwd lab toml) | Done (quarantine + forced `GZMO_CONFIG`) |
| Chaos-free Unix mentor on dedicated thread | Done (live `ping`/`teach`; wiring on git) |
| Discovery preflight → living mentor ping | Done (skills script) |
| Product smoke includes mentor ping | Done — `scripts/ct101-living-smoke.sh` |

**Still open for a clean baseline (priority order)**

1. **Honeypot↔Qdrant drift** — closing via `scripts/ct101-embed-backfill-loop.sh` (mirror + sync). Target ≥70% (warn clear); prefer ≥90%.
2. *(done)* **Hourly smoke timers** — CT101 `ct101-living-smoke.timer` + workstation `gzmo-ct101-living-smoke.timer`.

**Closed this pass**

| Item | Notes |
|------|--------|
| Selfheal HEAL-3 / HEAL-1 | Living `.184` probe; skip obsolete tunnels; no full-tree `find` hang |
| Pi `mentor-client.ts` | Prefers `/opt/gzmo/data/gzmo_mentor.sock`; default root `/opt/gzmo/current` |
| Pedagogy toml | `low_tension_dialogue` / `tension_oscillation` **disabled** on CT101 (match chaos-free mentor) |
| CORE_INSIGHT | Re-authored for CT101 living paths (seed `/opt/gzmo/data/vault.db`) |
