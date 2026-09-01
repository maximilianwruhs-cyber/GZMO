# ADR-0012 — Hardware-adaptive immutable appliance

- **Decision status:** Accepted (2026-08-31)
- **Implementation status:** Not started
- **Supersedes:** Partial supersession of [ADR-0010](./ADR-0010-clean-sheet-onebox.md) (one-box appliance phases move to implementation plan)
- **Spec:** [2026-08-31-self-developing-living-database-design.md](./superpowers/specs/2026-08-31-self-developing-living-database-design.md) §§5–6, 8

## Context

Current GZMO model selection depends on host-specific scripts and filenames. Hardware claims were informal. The North Star requires a single physical appliance that earns capability profiles by measurement, boots from signed media, and keeps system slots immutable while durable state stays encrypted and recoverable.

## Decision

GZMO ships as a **hardware-adaptive immutable appliance**:

1. **Thor-first capability ladder.** Profiles are earned by measured qualification, never by SKU marketing labels:
   - Scout (not product): Pi-class / Orin Nano class — boot, inventory, bounded experiments.
   - Living-Min: Jetson AGX Orin 64 GB class — full Living roles except optional code-candidate generation.
   - Living-Reference: Jetson AGX Thor 128 GB — release qualification target with full staged-autonomy headroom.
   - Forge/portability: qualified x86-64 one-node path proving interfaces are not ARM-only.
2. **Installer / recovery courier.** Portable media is signed installer, recovery environment, and offline update courier. It may carry architecture-specific payloads; it never holds the sole copy of operator root keys, recovery keys, or living state.
3. **BootTrust A/B.** A stable `BootTrust` seam (`verify_boot`, `unlock_state`, `stage_bundle`, `activate_inactive`, `mark_success`, `rollback`) drives immutable measured `system-A` / `system-B` slots with automatic failed-slot rollback and last-known-good fallback.
4. **Encrypted internal NVMe layout.** Installed disk separates immutable system slots; encrypted `data` (PostgreSQL authority + audit); encrypted content-addressed `models`; encrypted quota-bound wipeable `candidates`; and last-known-good snapshots with restore receipts.
5. **HIR → CM → catalog → qualification.** BootDiscovery produces a Hardware Inventory Record; CapabilityCompiler emits a signed Capability Manifest; ModelSelector chooses only catalog-compatible artifacts; cold role qualification pins a Qualification Record before RuntimeSupervisor starts role-qualified local models.

Offline updates use a TUF-shaped trust model (role-separated metadata, digest-bound targets, snapshot anti-mix, expiry and monotonic watermark). Hardware lacking selected trust roots is reduced-assurance Scout and cannot silently claim Living.

## Invariants

- Capability claims require measured qualification records; missing roles are explicit degradation or fail-closed loss of Living claim.
- No cloud fallback for core paths; no cross-architecture single executable image pretending to be portable.
- System slot changes are signed, inactive-first, assessed, and reversible.
- Durable data and audit survive OS rollback; model/system binaries are digest-referenced, not embedded in authority rows.
- Independent role contracts (`embed`, `rerank`, `extract_verify`, `conversational`, `code_candidate`) are never waived by shared weights.

## Consequences

- Host-filename GPU heuristics and hard-coded model paths are non-authoritative and scheduled for deletion at cutover.
- Scout targets may boot and experiment but must not be marketed or reported as reduced Living product.
- Numeric performance thresholds live in signed profile policy produced by qualification, not in agent-editable product logic.
- ADR-0011 floors (one node, honest capability, reversible change, airgap) constrain every appliance path.

## Rejected alternatives

- Dual-product Lite/Living hardware SKUs.
- Cloud-assisted model download or inference as silent fallback.
- Treating advertised TOPS/RAM/SKU as sufficient Living proof.
- In-place mutable rootfs updates without inactive-slot staging and rollback.
- Single shared “LLM role” that collapses independent qualification contracts.

## Verification

- Clean install, tampered image/catalog rejection, wrong-architecture refusal, PCR mismatch recovery, interrupted update, and automatic failed-slot rollback (North Star §15.1).
- Thor reference plus CPU fallback; independent role qualification; explicit unsupported roles; no behavior change when networking is physically absent (§15.2).
- `scripts/adr-check.sh` confirms Accepted status and required decision headings for this ADR.
