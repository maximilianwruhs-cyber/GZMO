# GZMO Architecture Decision Index

Authority order:
1. ADR-0011 constitutional invariants
2. ADR-0012/0013/0014 narrow target architecture
3. Accepted current-runtime ADRs not yet superseded in implementation
4. Operational docs
5. Research and proposed records

ADR-0001/0002 were never issued in GZMO. Historical links to sibling
little-tools-lab records are provenance only and are non-authoritative.

Decision status: Proposed | Accepted | Rejected | Superseded  
Implementation status: Not started | In progress | Implemented | Retired

## Lineage table

| ADR | Title | Decision status | Implementation status | Superseded by | Notes |
|---|---|---|---|---|---|
| 0003 | One living instance, overnight metabolism first | Superseded | Implemented | [ADR-0011](./ADR-0011-self-developing-living-database.md) | One-writer invariant retained in 0011 |
| 0004 | Airgap living is the USP (one box, full metabolism) | Superseded | Implemented | [ADR-0011](./ADR-0011-self-developing-living-database.md) | Airgap/one-box invariant retained in 0011 |
| 0005 | Continuous upgrade flywheel outranks frozen topology | Superseded | Implemented | [ADR-0014](./ADR-0014-constitutional-evolution.md) | Continuous improvement retained under capability envelopes |
| 0006 | One vault owner, two socket clients | Accepted | Implemented | — | Current runtime owner path until target successor cutover |
| 0007 | One product: living Keep (no lite SKU) | Superseded | Implemented | [ADR-0011](./ADR-0011-self-developing-living-database.md) | One-product invariant retained in 0011 |
| 0008 | Edge SSM backbone + structured memory backend | Superseded | Not started | — | Superseded Proposal; SSM remains a catalog candidate; MemoryLake not selected |
| 0009 | pgvector Vault Consolidation | Superseded | Not started | [ADR-0013](./ADR-0013-authoritative-full-stack-data-plane.md) | pgvector spike retained as evidence |
| 0010 | Clean-Sheet One-Box Living Memory Prototype | Superseded | Not started | [ADR-0011](./ADR-0011-self-developing-living-database.md), [ADR-0012](./ADR-0012-hardware-adaptive-immutable-appliance.md), [ADR-0013](./ADR-0013-authoritative-full-stack-data-plane.md), [ADR-0014](./ADR-0014-constitutional-evolution.md) | Superseded before implementation; phases move to implementation plan |
| 0011 | Self-developing Living Database Constitution | Accepted | Not started | — | Constitutional invariants; highest authority |
| 0012 | Hardware-adaptive immutable appliance | Accepted | Not started | — | Boot/trust/capability ladder |
| 0013 | Authoritative full-stack data plane | Accepted | Not started | — | PostgreSQL+pgvector sole durable authority |
| 0014 | Constitutional evolution and promotion | Accepted | Not started | — | M/T/C/P/A tiers, evaluator, promotion |

## Vocabulary

- **Decision status** records whether the decision is binding.
- **Implementation status** records whether the current tree realizes that decision.
- A decision may stay **Accepted** while implementation is **Not started** (target architecture).
- A decision may be **Superseded** while historical implementation remains **Implemented** until cutover retires it.
- **ADR-0006** stays **Accepted / Implemented** for the current owner control plane until the North Star owner path ships and cuts over.
