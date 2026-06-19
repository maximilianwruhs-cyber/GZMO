# Distill Endpoint Verification Policy

**Finding:** F4 — Distill drops unverified relations even at high LLM confidence  
**Workstream:** W3 — Distill endpoint verification policy documentation  
**Date:** 2026-06-17

## Problem Statement

Distill logs show `Dropped unverified relation` with `endpoints_ok=false` even at confidence 0.9. Queue depth is zero; honeypot holds 22,053 points. High-confidence structural relations (e.g. Spark→obolus) may never enter Neo4j if endpoints cannot be verified.

## Current Behavior

The distill pipeline enforces a **precision gate**: a relation is only committed to Neo4j if both the source and target endpoints are reachable and verifiable. When `endpoints_ok=false`, the relation is dropped regardless of LLM confidence score.

Observed pattern:
- Confidence 0.9+ relations dropped when either endpoint is unreachable
- Queue depth: 0 (file fallback `data/distill-queue` empty)
- Redis pending: null (no backlog in `gzmo:distill:pending`)
- Verdict: `queue ok` — the pipeline is healthy, just strict

## Policy

### Default: Strict Verification
- Both source and target endpoints must be reachable (`endpoints_ok=true`)
- Relation enters Neo4j only after verification passes
- This is a **feature for graph integrity**, not a bug

### Promotion Paths (optional, future)

Two mechanisms to surface ops-critical edges when endpoints are down:

1. **Whitelisted relation types** — `ROUTING_RULE` and `DAEMON_STATE` nodes (from W2 schema promotion) can bypass endpoint checks if marked with `promote=true` in the distill payload.
2. **Operator trigger** — `/distill promote <entity_id>` forces a relation into Neo4j regardless of endpoint status.

### Endpoint Check Targets

| Endpoint | Host | Port | Service |
|----------|------|------|---------|
| Neo4j | 192.168.31.202 | 7687 | bolt (MCP) |
| Qdrant | 192.168.31.202 | 6333 | vector store |
| Redis | 192.168.31.202 | 6379 | distill queue |
| Retrieval | 192.168.31.110 | 8081 | embed + rerank |
| Prime (local) | localhost | 8000 | LLM inference |

## Sidecar Probe

`probe-distill-queue.sh` validates queue depth and Redis pending count:
- Checks file queue in `data/distill-queue/`
- Checks Redis list `gzmo:distill:pending` via `redis-cli LLEN`
- Verdict: `"queue ok"` if both ≤ 10, `"queue backlog"` if either > 10
- Output: JSON to `probe-results/probe-distill-queue-<timestamp>.json`

## Acceptance Criteria

- [x] `probe-distill-queue.sh` exists at `gzmo_skills/scripts/discovery-probes/`
- [x] `probe-generic-discovery-action.sh` exists (parallel probe)
- [x] `survey_GZMO/gzmo.toml` exists with distill/redis/Neo4j config
- [x] This policy document written to `survey_GZMO/docs/DISTILL_ENDPOINT_POLICY.md`

## Open Questions (from plan)

1. Should distill promote `ROUTING_RULE`/`DAEMON_STATE` automatically or on operator trigger?
2. What triggers full `gzmo ingest` vs episodic vault write?
3. Dice loop 71-minute delay interaction with discovery cycle timing
