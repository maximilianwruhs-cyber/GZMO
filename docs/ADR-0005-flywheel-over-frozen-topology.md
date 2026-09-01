# ADR-0005 — Continuous upgrade flywheel outranks frozen topology

- **Historical status:** Accepted (2026-07-21)
- **Decision status:** Superseded
- **Implementation status:** Implemented
- **Superseded by:** [ADR-0014](./ADR-0014-constitutional-evolution.md) (continuous improvement retained under capability envelopes)
- **Historical supersedes for process / topology / promotion:** conflicting bits of [ADR-0003](./ADR-0003-one-instance-metabolism.md), [ADR-0004](./ADR-0004-airgap-living-usp.md) (invariants retained), and [LTL ADR-0001](../../little-tools-lab/docs/adr/0001-two-stack-lab-not-ct101-graft.md) (provenance only; non-authoritative in GZMO)
- **Implements:** [CONTINUOUS_UPGRADE.md](./CONTINUOUS_UPGRADE.md)
- **Related:** uniqueness craft backlog · Brain Feed · beat-gate kit · [ADR-0007](./ADR-0007-one-product-living.md) (one product)

## Context

ADR-0003/0004 and LTL-0001 were written to stop dual-writers and brand drift. That worked — then the same text froze **where** living runs, **whether** a single beat-gated loop may land, and **how fast** craft from the uniqueness tier list can reach the living brain.

We now have:

1. A ranked uniqueness backlog (S/A kernels worth deepening).
2. A four-ring continuous upgrade flywheel ([CONTINUOUS_UPGRADE.md](./CONTINUOUS_UPGRADE.md)).
3. Working beat-gate / Brain Feed / soak machinery.

ADRs that block that flywheel are **wrong for the product**, not “more careful.”

## Decision

### Layer A — Invariants (keep; from 0003/0004)

These are physics. Do not “rethink” them away:

1. **One overnight writer per vault** — never two concurrent metabolisms on the same vault.
2. **Airgap honesty** — core recall/distill/dream must not require the public internet; cloud LLM is opt-in.
3. **No public multi-tenant MCP webserver SKU** — brand attach is stdio / localhost.
4. **~~Lite is bootstrap~~** — **superseded by [ADR-0007](./ADR-0007-one-product-living.md):** no lite SKU. `~/.gzmo` without overnight is not the product (and still must not become a second overnight writer).

### Layer B — Topology (mutex, not destiny)

**Living host is an ops choice**, not a permanent CT101 sentence.

| Mode | Overnight writer | Condition |
|------|------------------|-----------|
| `living=CT101` | CT101 `/opt/gzmo` daemon | Default reference today |
| `living=workstation` | Workstation `gzmo serve` / local data dir | **Allowed for development** when CT101 writers are stopped |
| `living=appliance` | Any one airgapped box | USP target |

**Mutex rule:** enabling overnight writers on host A requires stopping overnight writers on every other host that shares (or would race) that vault. Prefer `scripts/living-host-mutex.sh` (claim / release). Dual-writer checks remain FAIL, not soft advice.

ADR-0003’s “workstation is never living” placement is **amended**: workstation **may** be the living host during an explicit claim window.

### Layer C — Promotion (flywheel, not full-cutover theater)

1. **Promote-by-loop is allowed.** Beat-gate green for a single loop (`config|ops|cognition|knowledge|discovery|…`) authorizes a **narrow** handoff into the *current* living host after human ack — not a mythical full-stack cutover.
2. **LTL-0001 “CT101 frozen forever / full assembly only” is superseded.** Lab remains the extract/fixture home; living may absorb one proven loop at a time.
3. **Beat-gate green still ≠ silent deploy.** Operator ack (or `PROMOTE_LOOP=<name>` + dual-writer PASS) is required. `CUTOVER_APPROVED=1` remains for **whole-host** migration only.
4. **Soak gates Unpark theater**, not kernel development. S/A craft and beat-gate work proceed while soak is HOLD; only *theater Unpark waves* wait on `soak_ready_unpark_ok`.
5. **Arena / calibration stay suggest-by-default.** Fast-pin is allowed when the operator runs an explicit promote script; no silent overnight toml clobber.
6. **Unique kernels are first-class upgrade targets.** Ring 4 (monthly craft) is doctrine, not optional tourism — deepen S/A items even if they began life as LTL “lab-only” (LTL-0002 labels demote *scheduler cron*, not *code craft*).

### Layer D — ADR hierarchy

When docs conflict:

```text
ADR-0005 (flywheel) > CONTINUOUS_UPGRADE.md > older “never reverse” process language
Layer A invariants still win over any speed shortcut
```

## Consequences

- Update ADR-0003/0004 headers: invariants retained; topology/promotion defer to this ADR.
- Supersede LTL ADR-0001 with promote-by-loop.
- Rewrite CONTINUOUS_UPGRADE / UNPARK / SPINE “never reverse” lists to separate invariants vs process.
- Add `scripts/living-host-mutex.sh` for claim/release.
- Development velocity: claim workstation living → deepen kernels → beat-gate → promote loop → release mutex / restore CT101 as needed.

## Non-goals

- Allowing two overnight writers “for convenience.”
- Auto-promoting Arena champions without operator action.
- Making HSP/pantheon a GREEN overnight gate.
- Public MCP webserver SKU.
