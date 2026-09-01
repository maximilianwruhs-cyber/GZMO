# ADR-0013 — Authoritative full-stack data plane

- **Decision status:** Accepted (2026-08-31)
- **Implementation status:** Not started
- **Supersedes:** [ADR-0009](./ADR-0009-pgvector-vault.md); partial supersession of [ADR-0010](./ADR-0010-clean-sheet-onebox.md) (storage phases move to implementation plan)
- **Spec:** [2026-08-31-self-developing-living-database-design.md](./superpowers/specs/2026-08-31-self-developing-living-database-design.md) §9

## Context

Production memory today splits authority across SQLite, asynchronous Qdrant mirroring, Neo4j, and Redis queues. Measured drift and multi-roundtrip retrieval proved the split is operationally costly and correctness-fragile. ADR-0009’s pgvector spike is retained as evidence that PostgreSQL+pgvector can own hybrid recall, but the broader authority model must be decided before implementation.

## Decision

1. **PostgreSQL 16 + pgvector is the sole durable authority** for facts, quarantine, evidence, bi-temporal supersession, utility/outcome observations, durable ingest and work claims, entities/relations, model qualifications and pins, candidates/evaluations/promotions/rollbacks, energy/resource observations, transactional outbox events, and hash-linked audit events.
2. **Transactional outbox.** The one owner commits domain changes and monotonic outbox events in the same PostgreSQL transaction. Projection workers consume at least once and apply idempotently. Projected records carry authoritative entity version, event sequence, and source digest.
3. **Mandatory full-Living accelerators, correctness-neutral.** Qdrant (high-throughput vectors), Neo4j (graph reasoning), and Redis (hot cache, scratch, queue notification) are required for a fully qualified Living profile, but:
   - none owns durable truth;
   - none accepts direct product writes;
   - each publishes a durable watermark;
   - fast reads use a projection only when consistency policy accepts its watermark; otherwise fall back to PostgreSQL;
   - every Qdrant/Neo4j result is revalidated against PostgreSQL validity, evidence, and authorization before recall;
   - all three can be discarded and rebuilt from a PostgreSQL snapshot plus ordered outbox.
4. **Correctness fallback.** PostgreSQL FTS and pgvector form the complete correctness path. Exact vector search is baseline; HNSW enables only with measured corpus/latency evidence.

## Invariants

- One authoritative writer owns all durable transitions (ADR-0011); accelerators are projections or ephemeral acceleration.
- Reconciliation compares IDs, versions, and digests—not aggregate counts.
- Accelerator snapshots may speed recovery but are never required for correctness.
- No dual authoritative write during cutover; SQLite import is offline, validated, then atomic authority switch with signed read-only archive retention.

## Consequences

- SQLite-as-production-authority, `VaultBackend` / dead `QdrantVault` paths, direct Qdrant/Neo4j product writes, and Redis-owned durable queue assumptions are non-goals for the target plane.
- ADR-0009 remains historical evidence (spike GO) under Superseded decision status.
- Living status must expose PostgreSQL authority health, one-writer state, projection watermarks/lag, and rebuild progress.
- Degraded accelerator paths remain explicit: FTS+pgvector, entity SQL, or durable PG queue as specified in North Star §13.

## Rejected alternatives

- Keeping SQLite as SoT with mirrored Qdrant as co-equal retrieval truth.
- Making Qdrant, Neo4j, or Redis accept direct product writes.
- Count-ratio-only drift checks as correctness proof.
- Optional accelerators for full-Living profile claims (they are mandatory for full Living, still rebuildable).
- Dual-write cutover windows with two authoritative stores.

## Verification

- Evidence-linked promotion, lifecycle transitions, bi-temporal reads, forget/purge, utility learning, durable queue, and outbox atomicity (North Star §15.3).
- Kill, corrupt, lag, and rebuild Qdrant, Neo4j, and Redis independently while PostgreSQL fallback remains correct (§15.4).
- `scripts/adr-check.sh` validates this ADR’s Accepted status, supersession target existence, and required headings.
