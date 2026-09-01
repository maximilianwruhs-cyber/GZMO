# ADR-0011 — Self-developing Living Database Constitution

**Decision status:** Accepted (2026-08-31)
**Implementation status:** Not started
**Supersedes:** [ADR-0003](./ADR-0003-one-instance-metabolism.md), [ADR-0004](./ADR-0004-airgap-living-usp.md), [ADR-0007](./ADR-0007-one-product-living.md); partial supersession of [ADR-0010](./ADR-0010-clean-sheet-onebox.md) (phases move to implementation plan)
**Spec:** [2026-08-31-self-developing-living-database-design.md](./superpowers/specs/2026-08-31-self-developing-living-database-design.md) §§1, 4, 17

## Context

GZMO’s product insight—verify and promote memories into a living, air-gapped Keep—was correct, but ADR-0003–0010 mixed invariants, topology, product positioning, research spikes, and implementation plans. Active documents also leaned on inaccessible sibling ADR-0001/0002 records that GZMO never issued. Constitutional floors must be explicit, conjunctive, and authoritative before runtime automation expands.

## Decision

GZMO is a **Self-developing Living Database**: one physical edge node that discovers and qualifies hardware and local models, maintains evidence-grounded memory, and develops candidate improvements without acquiring authority over its own production safeguards.

The following invariants are **conjunctive hard floors**. No score or performance gain may compensate for violating one:

1. **One physical node.** Each installation is complete without a cloud service, remote inference host, or second database host. Local containers are allowed.
2. **One authoritative writer.** One owner runtime mediates all durable state transitions. Other processes submit intents or consume committed events.
3. **Runtime airgap.** Core boot, search, recall, extract, verify, promote, consolidate, evolve, evaluate, and recover paths require no public network.
4. **Evidence before memory.** Assertions reach durable memory only through extraction, verification, evidence binding, lifecycle classification, and promotion.
5. **No self-issued authority.** Candidates cannot modify or sign their evaluator, fitness floors, trust roots, capability envelopes, promotion verifier, audit root, or last-known-good state.
6. **Operator authority for high-blast changes.** Code, schemas, model binaries, security policy, and capability expansion require a detached operator signature.
7. **Reversible production changes.** Every production update has a verified last-known-good target and an exercised rollback path.
8. **Honest capability.** Missing hardware, model, trust, or accelerator capability is explicit. The appliance never silently falls back to cloud or claims a profile it did not qualify.
9. **Audit continuity.** Every candidate, evaluation, promotion, rejection, envelope change, slot change, and rollback is append-only and integrity-linked.
10. **One product.** Scout and degraded states do not create Lite editions or independent brains.

## Invariants

- One node, one writer, airgap honesty, evidence-before-memory, no self-issued authority, reversible change, honest capability, and one product are binding across all subsequent architecture ADRs.
- Scout is a development/recovery target, not a second SKU. Degraded is a runtime state of a previously qualified Living node.
- Sibling little-tools-lab ADR-0001/0002 links are provenance only and are non-authoritative in GZMO.

## Consequences

- ADR-0003/0004/0007 decision authority moves here; retained invariants are restated above rather than left as historical implication.
- Target architecture ADRs (0012–0014) may refine mechanism but cannot weaken these floors.
- Active entry docs must cite this constitution (and the ADR index), not inaccessible external ADRs.
- Current-runtime ADR-0006 remains Accepted/Implemented until its successor owner path cuts over.

## Rejected alternatives

- Treating topology freezes or host placement as constitutional peers to one-writer/airgap/one-product.
- Shipping a Lite / attach-only SKU as a second product brain.
- Allowing performance gains to compensate for missing evidence, dual writers, cloud fallback, or self-signed authority.
- Keeping inaccessible sibling ADR-0001/0002 as active GZMO authority.

## Verification

- ADR index lists 0011 first in authority order and records 0001/0002 non-issuance.
- Entry-point docs (`AGENTS.md`, `MACHINE.md`, `README.md`, `docs/SPINE_FOCUS.md`) do not depend on `little-tools-lab/docs/adr/000[12]-`.
- `scripts/adr-check.sh` validates required headings, Accepted decision status, and lineage targets.
- Release qualification must fail closed on dual-writer, airgap breach, missing evidence path, capability misclaim, or irreversible production change (North Star §15).
