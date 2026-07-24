# GZMO ecosystem cron — OpenClaw orchestration playbook

**Generated** — `scripts/sync-openclaw-workspace.sh`  
**Living overnight writer:** CT101 `gzmo-daemon` (`/opt/gzmo`)  
**Evolve plane:** workstation systemd user timers  
**Never:** start `gzmo-serve` while CT101 lives

## How THIS agent finds jobs

The OpenClaw **`cron` tool is DENIED** (`tools.deny`) so llama.cpp tool schemas stay healthy.  
Jobs still run in the gateway.

```bash
bash bin/list-gzmo-crons.sh
# also: CRON_JOBS.md (snapshot) · openclaw cron list
```

## Planes

| Plane | Schedule owner | Examples |
|-------|----------------|----------|
| Living metabolism | CT101 daemon TOML | dream 01:00, distill 02:15, spark 03:30/22:30, ripen 00:00 |
| Brain Feed satellite | CT101 timer | tinyfolder-overnight 02:45 |
| Evolve | workstation user timers | ops-health, research-scan, evolve-daily/weekly |
| Operator announce | OpenClaw gateway cron | morning-brief, dual-writer-guard, weekly-mission |

## Operator OpenClaw jobs (typical)

Command (quiet / failure-alert): dual-writer-guard, living-smoke, ops-health, research-inbox  
Announce digests: daily-evolve, weekly-evolve  
Agent briefs: morning-brief, spark-followup, weekly-mission  

Refresh snapshot after changes: `bash ~/github-clone/GZMO/scripts/sync-openclaw-workspace.sh`

## Hard rules

1. Prefer existing scripts/timers over inventing overnight writers  
2. Serendipity / Arena: suggest only — never auto-apply  
3. Opportunity ship: one active bet; human kickoff  
4. Report from `~/github-clone/GZMO/data-next/` artifacts when present  
