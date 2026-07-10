# GZMO True Potential — Corrected Plan

**Status:** Corrected (2026-07-08)
**Based on:** Live probes of the actual production system on CT101
**Architecture:** Pi (workstation, cloud) → Sidecar (CT101 daemon + stores, VM200 retrieval)

---

## Architecture

```
CLOUD (workstation, this session)       SIDECAR NETWORK (192.168.31.x)
┌──────────────────────────────┐       ┌──────────────────────────────────────┐
│  Pi (cloud LLM via OpenRouter)│       │  PVE Host .200 (i7-6770HQ)           │
│  gzmo mcp-serve (PID 159178) │       │  ├─ CT101 .202                       │
│  Local Prime (ornith-35b)    │       │  │  ├─ gzmo daemon ✅ RUNNING        │
│  SSH key: id_sidecar_proxmox │       │  │  ├─ Neo4j :7687 ✅                │
│                              │       │  │  ├─ Qdrant :6333 ✅               │
│                              │       │  │  ├─ Redis :6379 ✅                │
│                              │       │  │  └─ vault.db: 658 MB ✅           │
│                              │       │  └─ VM200 .110                       │
│                              │       │     ├─ Embed :8081 ✅                │
│                              │       │     └─ Rerank :8081 ✅ (consolidated)│
└──────────────────────────────┘       └──────────────────────────────────────┘
```

---

## Current State (Real — from CT101 probes)

### ✅ Working

| Component | Detail |
|---|---|
| **Daemon** | `gzmo-daemon.service` running on CT101 since 10:22 UTC (132 MB RSS) |
| **DREAMS.md** | 22,804 lines — nightly dream consolidation working |
| **vault.db** | 658 MB — 59,682 semantic facts, 46,824 honeypot entries |
| **Neo4j** | 13,169 entities, 62,193 relations |
| **Qdrant** | 24,083 points in `honeypot` collection |
| **Synapse events** | 389,808 logged events |
| **Episodic memory** | 30+ daily files (June 2 → July 8) |
| **SparkEngine** | Producing crystallized connections daily |
| **SessionDistill** | Correlating with Spark lineage |
| **Embed :8081** | Consolidated embed + rerank on VM200 |
| **Ingest dedup** | 559 entries — pipeline running |
| **SSH key** | ✅ `id_sidecar_proxmox` authorized on PVE + VM200 |

### 🟡 Quality Issues

| Issue | Detail | Impact |
|---|---|---|
| **Sys Janitor cadence** | Was every 30 min → **FIXED to every 6h** | Saves ~44 LLM calls/day |
| **CircleCI lightsaber** | 🔧 FIXED above | — |

### ❓ Needs Verification

| Question | How to Check |
|---|---|
| Is Distill dedup still path-based? | Check `distill_dedup` growth rate over next 48h |
| What's Recall@5 on the real vault? | Run `retrieval-probes.py` from CT101 |
| Are there any errors in daemon logs? | `journalctl -u gzmo-daemon --since "1 week ago" \| grep -i error` |
| Is the local vault on workstation needed? | No — CT101 has the real vault |

---

## What's Left to Do

### Immediate Fixes (done this session)
- [x] SSH key generated and authorized on PVE + VM200
- [x] Sys Janitor cadence reduced from 30 min → 6 hours
- [x] Deploy scripts updated for consolidated embed+rerank on :8081
- [x] SSH config created (`docs/ssh-sidecar-config`)
- [x] Plan corrected with real CT101 data

### Tomorrow Morning
- [ ] Check daemon logs after restart: `ssh pve "pct exec 101 -- journalctl -u gzmo-daemon --since '5 min ago'"`
- [ ] Verify Sys Janitor only fires 4×/day now: check after 24h
- [ ] Run a Distill quality check on CT101
- [ ] Check if Recall@5 needs tuning on the real vault data

### Backlog (Ceiling Roadmap M3–M5)
- [ ] Measure Distill dedup hit rate (path-based vs content-hash)
- [ ] Tune Recall@5 if below 0.50 (HyDE/MMR parameters)
- [ ] Work toward M3: Dream/Spark from honeypot instead of episodic
- [ ] Work toward M4: Continuous eval gate before ingest changes
- [ ] Work toward M5: Mature DB after months of curated operation

---

## Quick Reference

```bash
# Check daemon
ssh pve "pct exec 101 -- systemctl status gzmo-daemon"

# Tail daemon logs
ssh pve "pct exec 101 -- journalctl -u gzmo-daemon -f"

# Check vault stats
ssh pve "pct exec 101 -- python3 -c '
import sqlite3
c = sqlite3.connect(\"/opt/gzmo/data/vault.db\")
for t in c.execute(\"SELECT name FROM sqlite_master WHERE type=\\\"table\\\"\"):
    n = t[0]
    cnt = c.execute(f\"SELECT COUNT(*) FROM \\\"{n}\\\"\").fetchone()[0]
    print(f\"  {n}: {cnt}\")
c.close()
'"

# Verify embed+rerank
curl -s http://192.168.31.110:8081/v1/models

# Read latest dream
ssh pve "pct exec 101 -- tail -50 /opt/gzmo/DREAMS.md"
```

---

## Success Metrics (Corrected)

| Metric | Today | Target |
|---|---|---|
| vault.db size | 658 MB | Growing |
| Semantic facts | 59,682 | Growing with Distill |
| Honeypot entries | 46,824 | 10–30% of vault |
| DREAMS.md | 22,804 lines | Nightly |
| Sys Janitor | 4×/day (was 48×) | Anomaly-only ideal |
| SSH access | ✅ PVE + VM200 | Maintained |