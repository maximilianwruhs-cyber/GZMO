# GZMO-next Gap Analysis — CT101 vs Workstation

**Date:** 2026-07-15  
**Source:** CT101 Capabilities Overview + GZMO-next config review  
**Purpose:** Document what's missing on GZMO-next to reach feature parity with CT101

---

## Executive Summary

| Metric | CT101 | GZMO-next |
|--------|-------|-----------|
| Systems 10-120 | 12/12 active | 2 fully, 6 partial, 2 missing |
| Cognition loops | 4/4 running | 3/6 running (Dream, Spark, Distill ✓; Ingest, Wiki, KG-Reconcile ✗) |
| Memory pipeline | 60k facts, 37k honeypot | 94 facts, 0 honeypot |
| Observability | Heartbeat + 488k events | One-shot ops-smoke |
| Self-improvement | Discovery cycles live | Not implemented |

---

## System-by-System Gap Matrix

| # | System | CT101 Status | GZMO-next Status | Gap |
|---|--------|-------------|------------------|-----|
| 10 | **Host & Runtime** | ✅ Production | ⚠️ Partial | No container orchestration automation; no sidecar auto-restart policy |
| 20 | **Daemon Core** | ✅ Production | ⚠️ Partial | `ops-smoke.sh` only one-shot at startup — no recurring heartbeat, no file watchers, no triage pipeline |
| 30 | **Cognition Engines** | ✅ Production | ⚠️ Partial | Missing: Ingest engine (`enabled=false`), Wiki engine, KG Reconcile, `knowledge-smoke.sh` |
| 40 | **LLM Gateway** | ✅ Production | ⚠️ Partial | `active_mode=local`, no cloud failover, no Obolus routing, no token metering |
| 50 | **Memory Data Plane** | ✅ Production | ⚠️ Partial | No Honeypot Promotion Cron, no Evidence Pipeline, no Redis Scratch Writer, `qdrant-vault-sync.sh` not in lab |
| 60 | **Chaos Engine** | ✅ Production | 🔴 Missing | No `chaos-pulse.sh`, no Thought-Cabinet, no tension-triggered discovery |
| 70 | **MCP Layer** | ✅ Production | ⚠️ Partial | No MCP Manager Bridge, no Pi Vault Bridge |
| 80 | **Synapse Bus** | ✅ Production | ⚠️ Partial | No JSONL rotation script, no session pull loop |
| 90 | **Tools & Skills** | ✅ Production | ⚠️ Partial | No Skill Discovery API, no Chat-Only Delegation |
| 100 | **Discovery Automation** | ✅ Degraded | 🔴 Missing | No Auto-socratic Pi-Cycles, no Discovery Remediation Queue |
| 110 | **External Nodes** | ✅ Production | ✅ Implemented | Prime + OKForge Observatory (`:3000/observatory`) |
| 120 | **Two-Stack Boundary** | ✅ Policy | ✅ Implemented | `assembly`-Guard enforced |

---

## Priority: P0 — Core Pipeline (highest impact)

| # | Enhancement | Recipe | Impact |
|---|-------------|--------|--------|
| 1 | **Wiki-Engine** | `wiki-okforge-push.sh` + `gzmo wiki push` | **Done (OKForge-backed)** — concepts land in `gzmo/gzmo-next-memory` after distill/dream/catch-up |
| 2 | **Ingest-Engine** | `ingest-smoke.sh` | Knowledge-folder watcher, verified fact extraction, source page emission |
| 3 | **Honeypot Promotion** | `promote-all-mature.py` cron | Move facts from vault → honeypot, enable semantic recall via Qdrant |
| 4 | **KG Reconcile** | `kg-reconcile.sh` | Sync verified entities/relations from vault to Neo4j graph |

---

## Priority: P1 — Observability & Infrastructure

| # | Enhancement | Recipe | Impact |
|---|-------------|--------|--------|
| 5 | **Recurring Heartbeat** | `heartbeat-loop.sh` | Replace one-shot ops-smoke with continuous health monitoring |
| 6 | **Qdrant Sync Recipe** | Copy `qdrant-vault-sync.sh` to lab | Nightly Qdrant sync at 01:45 (configured but recipe missing from lab) |
| 7 | **Synapse Rotation** | `synapse-rotate.sh` | JSONL rotation at 500k+ lines to prevent unbounded growth |
| 8 | **MCP Manager Bridge** | `mcp-manager-bridge.sh` | Neo4j child retry on crash, stable MCP connection |
| 9 | **Obolus Integration** | `obolus-metering.sh` | Token/cost ledger, cloud failover, task→engine map |

---

## Priority: P2 — Self-Improvement (long-term)

| # | Enhancement | Recipe | Impact |
|---|-------------|--------|--------|
| 10 | **Chaos Engine** | `chaos-pulse.sh` + Thought-Cabinet | Lorenz pulse, thought cabinet, low-tension discovery triggers |
| 11 | **Discovery Automation** | `discovery-automation.sh` | Auto-socratic Pi-Cycle, sidecar-only remediation queue |
| 12 | **Skill Discovery API** | `skill-discovery.sh` | Observatory séance panel integration |

---

## Implementation Notes

### Lab Recipe Convention
- All recipes live in `little-tools-lab/scripts/`
- Scheduler invokes via `gzmo-scheduler` cron (see `gzmo-scheduler/src/jobs.rs`)
- Recipes must accept `--live` / `--fixture` mode
- Output written to `data-next/` or `/tmp/` with meta JSON

### Config Entries Needed
- `[wiki]` section: `enabled`, `directory`, `sync_cron_hour/min`, `lint_cron_dow/hour`
- `[ingest]` section: `enabled=true`, `watcher_path`, `extract_engine`
- `[promote]` section: `cron_hour/min`, `honeypot_min_confidence`
- `[chaos]` section: `pulse_cron_hour/min`, `thought_cabinet_enabled`
- `[discovery]` section: `enabled`, `pi_cron_schedule`, `remediation_queue`

### Testing
- Each recipe validated with `beat-gate.sh` comparing output to CT101 baseline
- `gzmo-scheduler` runs recipes via `run_lab_script()` in `spawn.rs`
- Meta JSON written to `/tmp/*-meta.json` for observatory consumption

---

## Current Status (2026-07-15)

| Feature | Status | Notes |
|---------|--------|-------|
| Session-Continuity | ✅ Implemented | Session-Summary + Discovery mechanism |
| Session Distillation | ✅ Implemented | Batch-mode, 36 facts extracted from 5 sessions |
| Dream Consolidation | ✅ Implemented | `session-to-dream.sh` batch + vault promotion |
| Knowledge Graph | ❌ Empty | 0 entities, 0 relations — needs KG Reconcile |
| Honeypot | ❌ Empty | No promotion pipeline |
| Wiki | ✅ OKForge | `backend = "okforge"` → `gzmo/gzmo-next-memory` |
| Ingest | ❌ Disabled | `ingest.enabled = false` in config |
| Chaos Engine | ❌ Not implemented | No pulse recipe |
| Discovery | ❌ Not implemented | CT101-only feature |

---

*Last updated: 2026-07-15*
