# CT101 boundary — standalone legacy

**Status:** Accepted (2026-07-10)  
**Supersedes:** [CT101_PROMOTION.md](./CT101_PROMOTION.md) (per-loop promotion — **retired**)

---

## Decision

**CT101 is a standalone legacy GZMO deployment.** Little Tools Lab does **not** swap individual daemon loops into CT101.

| | CT101 | Little Tools Lab |
|---|--------|------------------|
| **What it is** | Production GZMO today (`gzmo daemon`, gzmo-core inline) | Home for 46 pieces + recipes |
| **Lab integration** | **None** — no `[assembly]` flags, no subprocess graft | Build **GZMO-next** as full assembly |
| **beat-gate** | Reference baseline only | Proves lab recipes match or beat legacy behavior |
| **Cutover** | Replaced only when **entire** new stack is ready | Not loop-by-loop |

---

## What we do on CT101

- Keep `gzmo-daemon` running as today
- Ops: `systemctl`, journalctl, config hotfixes for **legacy** issues only
- Do **not** edit CT101 `gzmo.toml` to point loops at lab scripts
- Do **not** restart daemon for lab recipe promotion

---

## What we do on the workstation

- Develop and test pieces in `github-clone/<tool>/`
- Run `gzmo assemble <recipe>` and `test-all-little-tools.sh`
- Run `beat-gate.sh` to compare lab output vs legacy expectations
- Use `gzmo chat` as operator frontend (local distill enqueue, slash → assemble)
- Apply `gzmo-handoff --apply` to **workstation** `GZMO/config/gzmo.toml` when calibrating locally

---

## GZMO-next cutover (future)

When the **full** lab assembly is stack-ready (S3 in [LAB_TREATMENT.md](../../little-tools-lab/docs/LAB_TREATMENT.md)):

1. Document the new runtime (composed recipes + any lab-native daemon)
2. Prove live beat-gates on cognition, knowledge, ops, config as one unit
3. Plan a **single** migration to replace CT101 — not incremental loop swaps

Until then, CT101 and lab development proceed **in parallel**.

---

## References

- [LAB_TREATMENT.md](../../little-tools-lab/docs/LAB_TREATMENT.md)
- [PI_FRONTEND_SPLIT.md](./PI_FRONTEND_SPLIT.md) — topology (daemon on CT101)
- [OPERATOR_FRONTEND_DECISION.md](./OPERATOR_FRONTEND_DECISION.md) — gzmo_cli on workstation

---

*End.*
