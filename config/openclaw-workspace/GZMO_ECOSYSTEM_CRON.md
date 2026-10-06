# GZMO ecosystem cron — OpenClaw orchestration playbook

**Generated** — `scripts/sync-openclaw-workspace.sh`  
**Living overnight writer:** workstation `gzmo-daemon` (`~/.gzmo/`)  
**Evolve plane:** workstation systemd user timers  
**Inference:** vLLM dual-Blackwell `vllm-blackwell-prime` `:8000`  
**Never:** second overnight writer · do not assume CT101

## How THIS agent finds jobs

The OpenClaw **`cron` tool is DENIED** (`tools.deny`) so local tool schemas stay bounded.  
Jobs still run in the gateway.

```bash
bash bin/list-gzmo-crons.sh
# also: CRON_JOBS.md (snapshot) · openclaw cron list
```

## Planes

| Plane | Schedule owner | Examples |
|-------|----------------|----------|
| Living metabolism | workstation `gzmo-daemon` | dream / distill / spark / ripen (daemon TOML) |
| Brain Feed satellite | workstation timers | tinyfolder-overnight (if enabled) |
| Evolve | workstation user timers | ops-health, research-scan, evolve-daily/weekly |
| Operator announce | OpenClaw gateway cron | morning-brief, dual-writer-guard, weekly-mission |
| Inference keep-alive | systemd user | `vllm-blackwell-prime.service`, `openclaw-gateway.service` |

## Operator OpenClaw jobs (typical)

Command (quiet / failure-alert): dual-writer-guard, living-smoke, ops-health, research-inbox  
Announce digests: daily-evolve, weekly-evolve  
Agent briefs: morning-brief, spark-followup, weekly-mission  

Refresh snapshot after changes: `bash ~/github-clone/GZMO/scripts/sync-openclaw-workspace.sh`

## Hard rules

1. **One overnight writer** — never start `gzmo-serve` while living daemon owns the vault  
2. Operator **announces**; living **metabolizes**; evolve **researches/ships**  
3. No Qdrant/Neo4j writes from chat  
4. Prefer evidence from MCP / `data-next/` / `CRON_JOBS.md` over guesses  
5. Local replies cost VRAM — keep turns tight; `/new` after long threads
