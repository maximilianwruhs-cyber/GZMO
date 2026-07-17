# CT101 boundary — living production host

**Status:** Accepted (2026-07-10); production cutover 2026-07-15; **restored living 2026-07-17**  
**Supersedes:** [CT101_PROMOTION.md](./CT101_PROMOTION.md) (per-loop promotion — **retired**)  
**Restore runbook:** [CT101_RESTORE_LIVING.md](./CT101_RESTORE_LIVING.md)

---

## Decision

**CT101 is the sole living metabolism instance** (`gzmo daemon` + `/opt/gzmo/` vault). **The workstation is operator frontend + Prime fallback** (ADR-0003 amended 2026-07-17). Little Tools Lab does **not** swap individual daemon loops into CT101. Never run workstation `gzmo serve` overnight alongside CT101.

| | CT101 (living) | Workstation (operator / lab) |
|---|--------|------------------|
| **What it is** | **Production** — `gzmo.toml` + `gzmo daemon`, vault ~60k facts, Docker sidecars | Operator UI + Prime `:8000`; `config/gzmo.toml` → `data-next/` is **lab/dev scratch** |
| **Lab integration** | **None** — no lab recipe grafts | Lab recipes = beat-gate fixtures; optional `gzmo-scheduler` / `gzmo-serve` for parity only |
| **beat-gate** | Live baseline to beat | S2 gate on workstation stack before any future re-promotion |
| **Ops** | `ssh ct101` or `ssh pve "pct exec 101 -- …"`; `systemctl … gzmo-daemon` | CLI/chat/MCP against local clone; **do not** enable overnight `gzmo-serve` while CT101 lives |

---

## What we do on CT101 (production)

- Run and maintain `gzmo-daemon.service` + Redis/Qdrant/Neo4j sidecars
- Edit `/opt/gzmo/gzmo.toml` only for living ops (restart daemon after)
- Do **not** point CT101 `gzmo.toml` loops at lab scripts (`[assembly]=lab` grafts forbidden)

---

## What we do on the workstation (operator / lab)

- **`gzmo` / `gzmo chat`** — operator frontend (dev clone)
- **`llama-prime`** — local cognition at `:8000` (CT101 cloud-first fallback)
- **`gzmo memory mcp`** — lab MCP surface over `data-next/` (not the production vault)
- **`gzmo-serve` / `gzmo-scheduler`** — **disabled by default** after 2026-07-17 restore; enable only for explicit lab/beat-gate sessions with CT101 overnight writers stopped
- Local Qdrant/Redis sidecars — lab volumes only
- Observatory over `data-next/` is a lab viewer, not production control plane

---

## History

### GZMO-next cutover (completed 2026-07-15, reversed 2026-07-17)

Workstation briefly became sole living instance with fresh `data-next/` (no vault import from CT101). That placement was reversed on 2026-07-17 — see [CT101_RESTORE_LIVING.md](./CT101_RESTORE_LIVING.md).

---

## References

- [CT101_RESTORE_LIVING.md](./CT101_RESTORE_LIVING.md) — restore checklist + health commands
- [CT101_INFRASTRUCTURE_REPORT.md](./CT101_INFRASTRUCTURE_REPORT.md) — live-verified ecosystem map
- [ct101-systems/00-CAPABILITIES_OVERVIEW.md](./ct101-systems/00-CAPABILITIES_OVERVIEW.md)
- [ADR-0003-one-instance-metabolism.md](./ADR-0003-one-instance-metabolism.md)
- [PLACEMENT_DECISION.md](./PLACEMENT_DECISION.md)
- [PI_FRONTEND_SPLIT.md](./PI_FRONTEND_SPLIT.md)
- [OPERATOR_FRONTEND_DECISION.md](./OPERATOR_FRONTEND_DECISION.md)

---

*End.*
