# ADR-0003 — One living instance, overnight metabolism first

**Status:** Accepted (2026-07-16); **host placement amended 2026-07-17**  
**Supersedes for operator roadmap:** CT101↔next dual-stack as a permanent product shape  
**Related:** [CT101_BOUNDARY.md](./CT101_BOUNDARY.md), [CT101_RESTORE_LIVING.md](./CT101_RESTORE_LIVING.md), [ARCHITECTURE_GZMO_PLATFORM.md](./ARCHITECTURE_GZMO_PLATFORM.md)

## Context

GZMO grew a dual stack (CT101 legacy + GZMO-next), many named engines, and a lab recipe zoo. The product claim — overnight memory that compounds — was outrun by parity organs (wiki, KG, chaos daemon path, discovery).

On 2026-07-15/16 production briefly moved to the workstation (`gzmo serve` + `data-next/`). On **2026-07-17** that cutover was reversed: CT101 resumed as the sole living metabolism brain (see [CT101_RESTORE_LIVING.md](./CT101_RESTORE_LIVING.md)).

## Decision

1. **One living instance only** — never two overnight writers. Dual-stack forever is not the product shape.
2. **Living host (amended 2026-07-17): CT101** — Rust `gzmo daemon` / `gzmo-daemon.service` under `/opt/gzmo/`, vault + Docker sidecars colocated. `active_mode=cloud` with Prime fallback on the workstation.
3. **Workstation is operator + Prime fallback** — `gzmo` / `gzmo chat`, Prime `:8000`, Cursor/Pi. Workstation `gzmo serve` / `data-next/` are **lab/dev scratch**, not production metabolism. Keep `gzmo-serve.service` **disabled** unless an explicit lab session requires it (and then stop CT101 daemon writers first — never both).
4. **Do not graft lab loops into CT101** — Little Tools Lab remains beat-gate fixtures; no `[assembly]=lab` pointing CT101 at lab scripts.
5. **Chaos is opt-in for chat** on lab/workstation configs. CT101 daemon path keeps its own chaos policy.
6. **Product gate:** CT101 daemon health (systemd + journal + vault/honeypot counts + Docker sidecars) — not Observatory as a second control plane. Workstation `gzmo status` over `data-next/` is lab-only.

## Consequences

- Lab recipes remain beat-gate fixtures, not the long-term production brain.
- `gzmo-scheduler` and `gzmo-serve` on the workstation stay offline by default after the 2026-07-17 restore.
- Graph/wiki/chaos enhancements land on CT101 only with explicit ops approval; no silent dual-writer reintroduction.
