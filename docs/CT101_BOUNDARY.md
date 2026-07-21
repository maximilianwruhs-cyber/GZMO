# CT101 boundary — reference living host

**Status:** Accepted (2026-07-10); restored living 2026-07-17; **amended by [ADR-0005](./ADR-0005-flywheel-over-frozen-topology.md) (2026-07-21)**  
**Restore runbook:** [CT101_RESTORE_LIVING.md](./CT101_RESTORE_LIVING.md)  
**Flywheel:** [CONTINUOUS_UPGRADE.md](./CONTINUOUS_UPGRADE.md)

---

## Decision

**CT101 is the default reference living host** (`gzmo daemon` + `/opt/gzmo/` vault) when `living-host-mutex` claim is `ct101` or unset in production ops.

**Workstation may claim living** for development (`claim --host workstation`) after CT101 overnight writers are stopped. Never two overnight writers at once ([ADR-0005](./ADR-0005-flywheel-over-frozen-topology.md)).

**Promote-by-loop is allowed** — beat-gate PASS for a single loop + operator ack may hand off that loop into the *current* living host ([LTL ADR-0003](../../little-tools-lab/docs/adr/0003-promote-by-loop.md)). Whole-host cutover still requires `CUTOVER_APPROVED=1`.

| | CT101 (reference living) | Workstation |
|---|--------|------------------|
| **Default role** | Production living when claimed | Operator UI + Prime; **dev living** when claimed |
| **Lab integration** | Narrow promote-by-loop after beat-gate + ack | Lab recipes, beat-gate kit, uniqueness craft |
| **beat-gate** | Live baseline / optional live smoke | Fixture (+ live) prove before promote |
| **Ops** | `ssh ct101` / `systemctl … gzmo-daemon` | `living-host-mutex.sh`; stop serve while CT101 claimed |

---

## What we do on CT101 (when it holds the claim)

- Overnight metabolism (distill, dream, spark, immune, …) per living toml.
- Accept **narrow** promote-by-loop diffs after beat-gate + ack — not silent CI grafts.
- Refuse dual-writer: workstation `gzmo-serve` / scheduler must be inactive while CT101 writes.

## What we do on workstation

- Operator frontend, Prime fallback, lab/dev, beat-gates.
- **Dev living window:** `bash scripts/living-host-mutex.sh claim --host workstation` → stop CT101 writers → run prove/promote → `release` → restore CT101 if desired.
- Brain Feed side-effects target whichever host currently holds the living claim.

## Related

- [ADR-0005-flywheel-over-frozen-topology.md](./ADR-0005-flywheel-over-frozen-topology.md)
- [ADR-0003-one-instance-metabolism.md](./ADR-0003-one-instance-metabolism.md)
- [CONTINUOUS_UPGRADE.md](./CONTINUOUS_UPGRADE.md)
- [BRAIN_FEED.md](./BRAIN_FEED.md)
