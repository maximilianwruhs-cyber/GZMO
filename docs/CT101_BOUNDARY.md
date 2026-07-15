# CT101 boundary — standalone legacy

**Status:** Accepted (2026-07-10); production cutover 2026-07-15  
**Supersedes:** [CT101_PROMOTION.md](./CT101_PROMOTION.md) (per-loop promotion — **retired**)

---

## Decision

**CT101 is a standalone legacy GZMO deployment.** **GZMO-next on the workstation is production** (since 2026-07-15). Little Tools Lab does **not** swap individual daemon loops into CT101.

| | CT101 | GZMO-next (workstation) |
|---|--------|------------------|
| **What it is** | Frozen legacy (`gzmo daemon`, gzmo-core inline) | **Production** — `gzmo-scheduler` + lab recipes |
| **Lab integration** | **None** — no `[assembly]` flags | `[assembly] = "lab"` via `GZMO_INSTANCE=next` |
| **beat-gate** | Historical reference baseline | S2 gate before trusting production |
| **Ops** | Leave alone unless explicitly debugging legacy | `systemctl --user` services, `data-next/` paths |

---

## What we do on CT101

- **Do not modify** for GZMO-next cutover — CT101 may run independently
- Legacy ops only when explicitly debugging CT101 itself
- Do **not** edit CT101 `gzmo.toml` to point loops at lab scripts

---

## What we do on the workstation (production)

- **`gzmo-scheduler`** — cron runner (`GZMO_INSTANCE=next`, `config/gzmo-next.toml`)
- **`llama-prime`** — local cognition at `:8000`
- **Sidecars** — local Qdrant + Redis (`database-cluster/`, user systemd)
- **`gzmo chat` / `gzmo assemble`** — operator frontend
- **Observatory** — `OBSERVATORY_MODE=local`, reads `data-next/`

---

## GZMO-next cutover (completed 2026-07-15)

Cutover steps executed:

1. Harden workstation systemd (linger, sleep masks, scheduler spark fix)
2. Local sidecars (Qdrant, Redis) — fresh volumes
3. Enable memory plane in `gzmo-next.toml` (VM200 embed/rerank, local Qdrant/Redis)
4. S2 beat-gate: config, ops, cognition, knowledge — all PASS
5. Observatory retargeted to local `data-next/`

CT101 remains frozen legacy reference; no vault import (fresh `data-next/`).

---

## References

- [CT101_INFRASTRUCTURE_REPORT.md](./CT101_INFRASTRUCTURE_REPORT.md) — live-verified ecosystem map (host, daemon, sidecars, code inventory)
- [ct101-systems/00-CAPABILITIES_OVERVIEW.md](./ct101-systems/00-CAPABILITIES_OVERVIEW.md) — capability matrix, advancement roadmap, per-subsystem reports
- [LAB_TREATMENT.md](../../little-tools-lab/docs/LAB_TREATMENT.md)
- [PI_FRONTEND_SPLIT.md](./PI_FRONTEND_SPLIT.md) — topology (daemon on CT101)
- [OPERATOR_FRONTEND_DECISION.md](./OPERATOR_FRONTEND_DECISION.md) — gzmo_cli on workstation

---

*End.*
