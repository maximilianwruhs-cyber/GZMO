# Controller Status Report

**Controller:** AI Agent (pi) — elevated to system controller  
**Date:** 2026-06-03T17:53 UTC  
**Authority:** User mandate — full operational control of GZMO infrastructure

---

## 1. Current State (2026-06-03 15:53 UTC)

| Component | Status | PID | Notes |
|-----------|--------|-----|-------|
| Prime (`:8000`) | ✅ | 3363 | Qwen3.6-35B-A3B, ctx 131072, layer-split on 2× 5070 Ti |
| Embed WS (`:8002`) | ✅ | 3362 | Qwen3-Embedding-0.6B, 1024-dim |
| GZMO daemon | ✅ | 24986 | **Rebuilt binary** (16:49), restarted 15:53 UTC |
| Neo4j MCP | ✅ | 8474 | LXC101 bolt |
| VM200 embed (`:8081`) | ✅ | — | Qwen3-Embedding-0.6B-Q8 |
| VM200 rerank (`:8082`) | ✅ | — | bge-reranker-v2-m3-Q8 |
| VM200 librarian (`:8083`) | ✅ | — | Qwen2.5-1.5B |
| Qdrant (`:6333`) | ✅ | — | honeypot = 682 points |
| Sovereign (`:8010`) | ⏸ | — | Parked, expected down |

### Memory Counts
| Layer | Count |
|-------|-------|
| semantic_vault | 2811 |
| honeypot (latest) | 682 |
| honeypot_fts | 682 |
| Qdrant honeypot | 682 |
| Qdrant knowledge | 3245 (legacy) |

### Resources
| Metric | Value | Status |
|--------|-------|--------|
| CPU | ~2.7% | ✅ |
| RAM | 34.2% (20.7/60.6 GB) | ✅ |
| Disk / | 12.9% (1595 GB free) | ✅ |

---

## 2. Issues Found & Fixed

### 2.1 CRITICAL: Stale daemon binary
- **Problem:** Daemon PID 3364 was running a pre-rebuild binary (started 16:41, binary rebuilt 16:49)
- **Impact:** DreamEngine, SparkEngine, SessionDistill, and Qdrant sync loops were NOT executing
- **Root cause:** Binary was rebuilt while daemon was still running the old version
- **Fix:** Killed stale daemon, restarted with new binary at 15:53 UTC
- **Verification:** `gzmo health` — all subsystems online; daemon logs show "All subsystems online — entering daemon loop"

### 2.2 BUG: verify-production.sh Qdrant sync dry-run false negative
- **Problem:** `grep -q 'facts with'` failed because Python script outputs `"facts (honeypot) with"`
- **Fix:** Changed grep pattern from `'facts with'` to `'facts.*with'` (line 121 of verify-production.sh)
- **Verification:** `./scripts/verify-production.sh` now exits 0

### 2.3 OPERATIONAL: Scheduled engines not yet fired (expected)
- **Current time:** 15:53 UTC
- **Next scheduled runs:**
  - **SparkEngine** at 22:30 UTC (~6.5 hours)
  - **DreamEngine** at 01:00 UTC (~9 hours)
  - **Qdrant sync** at 01:45 UTC (~9.8 hours)
  - **Session distill** at 02:15 UTC (~10.3 hours)
- **Status:** Engines are running in background tokio tasks; waiting for cron windows

---

## 3. Controller Actions Taken

1. ✅ **Full production verification** — `verify-production.sh` now passes (17/17 checks)
2. ✅ **Fixed grep pattern** in `scripts/verify-production.sh` (line 121)
3. ✅ **Restarted daemon** with fresh binary (PID 24986)
4. ✅ **Captured operational baseline** (memory counts, system resources, service status)
5. ✅ **Recorded in WORKING_MEMORY.md**

---

## 4. Upcoming Operations

| Time (UTC) | Engine | Action |
|------------|--------|--------|
| ~22:30 | SparkEngine | Serendipity hypothesis + verify |
| ~01:00 | DreamEngine | Episodic → vault + Neo4j |
| ~01:45 | Qdrant sync | honeypot collection sync |
| ~02:15 | Session distill | Sessions → vault facts |
| */30 | sys_janitor | Health monitoring (continuous) |

---

## 5. Open Items

| Item | Priority | Notes |
|------|----------|-------|
| Daemon restart procedure | Medium | Rebuild → restart daemon to avoid stale binary |
| Log rotation | Low | daemon.log 2458 lines, no rotation config |
| Journalctl capture | Medium | Daemon logs not reaching systemd journal (noisy stdout) |
| Qdrant `knowledge` cleanup | Low | 3245 legacy vectors — needs cutover checklist |

---

*This document is maintained by the controller. Update after significant events.*
