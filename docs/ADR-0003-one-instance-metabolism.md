# ADR-0003 — One living instance, overnight metabolism first

**Historical status:** Accepted (2026-07-16); host placement amended 2026-07-17; **process/topology superseded in part by [ADR-0005](./ADR-0005-flywheel-over-frozen-topology.md) (2026-07-21)**
**Decision status:** Superseded
**Implementation status:** Implemented
**Superseded by:** [ADR-0011](./ADR-0011-self-developing-living-database.md) (one-writer invariant retained)
**Related:** [ADR-0005](./ADR-0005-flywheel-over-frozen-topology.md), [ADR-0004](./ADR-0004-airgap-living-usp.md), [CT101_BOUNDARY.md](./CT101_BOUNDARY.md), [CONTINUOUS_UPGRADE.md](./CONTINUOUS_UPGRADE.md)

## Context

GZMO grew a dual stack (CT101 legacy + GZMO-next), many named engines, and a lab recipe zoo. The product claim — overnight memory that compounds — was outrun by parity organs. Dual overnight writers corrupted the vault story.

## Decision (invariants — still binding)

1. **One living instance only** — never two overnight writers on the same vault. Process lock: [ADR-0006](./ADR-0006-owner-control-plane.md) (`{vault_db}.write.lock` + owner socket).
2. **Product gate is living-host health** — systemd/journal/vault/honeypot/sidecars on whichever host currently holds the living claim — not Observatory as a second control plane.

## Amended (see ADR-0005)

| Was (2026-07-17) | Now (ADR-0005) |
|------------------|----------------|
| CT101 is permanently the only living host | CT101 is the **default reference**; living host is a **mutex claim** (`CT101` \| `workstation` \| `appliance`) |
| Workstation `gzmo serve` is never production metabolism | Workstation **may** be living during an explicit claim (CT101 writers stopped) |
| Do not graft lab loops into CT101 | **Promote-by-loop** allowed after beat-gate + operator ack into the *current* living host |
| Dual-stack forever is not the product | Still true — one writer; stacks are promote stages, not forever peers |

## Consequences

- Use `scripts/living-host-mutex.sh claim|release` when moving the overnight writer.
- Lab recipes feed the continuous upgrade flywheel; they are not banned from living after beat-gate.
- Chaos remains opt-in for chat unless a beat-gated cognition/config promote says otherwise.
