# OpenClaw workspace ↔ GZMO ecosystem contract

**Status:** Active (2026-10-06)  
**USP:** nutrient · Brain Feed · airgap living — OpenClaw is an **operator surface**, not a second overnight brain  
**Doctrine:** [ADR-0005](./ADR-0005-flywheel-over-frozen-topology.md) · [ADR-0004](./ADR-0004-airgap-living-usp.md) · CUTOVER A (CT101 → workstation living)

## One sentence

OpenClaw talks to Max on Telegram; **workstation living** (`gzmo-daemon`) metabolizes; the same workstation **evolves** and serves **vLLM dual-Blackwell** inference; living memory is reached only via MCP / takeaway — never a second brain.

## Plane map

```text
┌───────────────────── OPERATOR (OpenClaw) ─────────────────────┐
│  Telegram / workspace ~/.openclaw/workspace                   │
│  Search: MCP gzmo-living    Nutrient: openclaw-takeaway.sh    │
│  Model: vllm/Qwen/Qwen3.6-35B-A3B  (see MODELS.md)             │
│  Announce/orchestrate cron digests — does NOT own overnight   │
└───────────────┬───────────────────────────────┬───────────────┘
                │ read/search                   │ takeaway enqueue
                ▼                               ▼
┌──────────────── LIVING (workstation) ─────────┐
│  gzmo-daemon · ~/.gzmo · vault/honeypot       │
│  Redis/Qdrant/Neo4j sidecars (local)          │
│  dream/distill/spark/ripen — ONE overnight writer │
└───────────────────────────────────────────────┘
                ▲
                │ ops / research / opportunity (no second writer)
┌──────────────── EVOLVE + INFERENCE (workstation) ─────────────┐
│  systemd user timers: ops-health, research-scan, evolve-*     │
│  vllm-blackwell-prime.service :8000 (TP=2, AWQ, graphs)       │
│  openclaw-gateway.service (Telegram bot)                      │
│  gzmo-serve ONLY if living-host-mutex claimed                 │
└───────────────────────────────────────────────────────────────┘

Historical: CT101 (192.168.31.202) decommissioned — do not restore as living/inference host.
```

## File ownership in `~/.openclaw/workspace/`

| File | Owner | Sync? | Role |
|------|-------|-------|------|
| `ECOSYSTEM.md` | GZMO | **generated** | Plane map + never-list |
| `MODELS.md` | GZMO | **generated** | Local vLLM Prime / model ops |
| `LIVING_ATTACH.md` | GZMO | **generated** | Living MCP + takeaway contract |
| `GZMO_ECOSYSTEM_CRON.md` | GZMO | **generated** | Cron playbook |
| `CRON_JOBS.md` | GZMO sync | **generated snapshot** | Live OpenClaw + timer list |
| `TOOLS.md` | hybrid | ecosystem block **synced** | Host/SSH/MCP/inference facts |
| `TOOLS.local.md` | OpenClaw | **never overwrite if exists** | Cameras, TTS, personal nicknames |
| `AGENTS.md` | hybrid | ecosystem block **synced** | Rules; outer boilerplate kept |
| `SOUL.md` / `IDENTITY.md` | hybrid | thin ecosystem boundaries synced | Persona from packs or hand edit |
| `CHARACTER.md` | persona pack | never by sync | Optional overlay |
| `CHARACTER.active.json` | chooser | never by sync | Last selected pack slug |
| `USER.md` | hybrid | operator prefs synced | Max + GZMO working style |
| `HEARTBEAT.md` | OpenClaw | leave comments-only unless Max opts in | Empty = no heartbeat API spam |
| `memory/*.md` | OpenClaw | never | Daily scratch |
| `MEMORY.md` | OpenClaw | never | Curated long-term (main session) |
| `bin/*` | GZMO installers | synced/linked | `list-gzmo-crons`, `openclaw-takeaway` |

Markers in hybrid files:

```html
<!-- GZMO:ECOSYSTEM:BEGIN -->
…generated…
<!-- GZMO:ECOSYSTEM:END -->
```

Hand-edits **inside** markers are overwritten on next `sync-openclaw-workspace.sh`.  
Edit outside markers, use `TOOLS.local.md`, or change sources under `config/openclaw-workspace/`.

**Precedence:** generated full files (`ECOSYSTEM.md`, `MODELS.md`, …) always win over prior workspace copies. Marker blocks always win over hand-pastes inside the same markers. `TOOLS.local.md` and `MEMORY.md` / `memory/*` never lose to sync.

## Inference contract (vLLM)

| Item | Required value |
|------|----------------|
| OpenClaw primary | `vllm/Qwen/Qwen3.6-35B-A3B` |
| baseUrl | `http://127.0.0.1:8000/v1` |
| apiKey | `vllm-local` |
| Unit | `vllm-blackwell-prime.service` |
| Profile | `~/Projects/vllm-blackwell-backend/profiles/prime-fast.env` |
| contextWindow | ≥ 32768 (tool schemas alone exceed 16k) |

Retired as primary: `llamacpp/*`, `llama-prime.service`, key `llamacpp-local`.

Sync may optionally sanity-check `~/.openclaw/openclaw.json` (env `OPENCLAW_PATCH_MODELS=1` to auto-fix).

## Never (all surfaces)

1. Second overnight writer (`gzmo-serve` while living daemon owns vault)  
2. Qdrant upsert / Neo4j auto-graph from chat  
3. `session close --now` while living owns metabolism  
4. Claim “no cron jobs” without `bin/list-gzmo-crons.sh` / `CRON_JOBS.md`  
5. Treat OpenClaw as the GZMO product brain ([MACHINE.md](../MACHINE.md))  
6. Restore CT101 or llama.cpp as the live living/inference plane without an explicit new cutover  
7. Drop OpenClaw context below what tool schemas require (OOM / 400 context_overflow)

## Character packs (persona)

Use character packs **only through**:

```bash
bash scripts/openclaw-choose-character.sh --list
bash scripts/openclaw-choose-character.sh glados
```

**Telegram:** `/character list` · `/character search duck` · `/character glados` · `/character who`

Install once: `bash scripts/install-openclaw-character.sh`

Do **not** add Telegram `customCommands` for `/character`. Re-run workspace sync after pack installs.

## Sync command

```bash
bash scripts/sync-openclaw-workspace.sh
OPENCLAW_PATCH_MODELS=1 bash scripts/sync-openclaw-workspace.sh  # warn/fix openclaw.json primary
```
