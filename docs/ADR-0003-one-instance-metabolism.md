# ADR-0003 — One living instance, overnight metabolism first

**Status:** Accepted (2026-07-16)  
**Supersedes for operator roadmap:** CT101↔next dual-stack as a permanent product shape  
**Related:** [CT101_BOUNDARY.md](./CT101_BOUNDARY.md), [ARCHITECTURE_GZMO_PLATFORM.md](./ARCHITECTURE_GZMO_PLATFORM.md)

## Context

GZMO grew a dual stack (CT101 legacy + GZMO-next), many named engines, and a lab recipe zoo. The product claim — overnight memory that compounds — was outrun by parity organs (wiki, KG, chaos daemon path, discovery).

## Decision

1. **CT101 is a frozen reference machine** — not a forever architecture. Do not graft loops into it; do not treat “systems 10–120 parity” as the workstation roadmap.
2. **Workstation is the sole living instance** — one config spine (`config/gzmo.toml` → same paths as `gzmo-next.toml` / `data-next/`), one systemd unit for metabolism (`gzmo serve`).
3. **Three surfaces only for daily ops:**
   - `gzmo` / `gzmo chat` — operator REPL (`AgentSession`)
   - `gzmo serve` — thin cron + queue workers; typed Rust overnight jobs
   - `gzmo memory mcp` (alias of `gzmo mcp-serve`) — MCP memory tools for Cursor/Pi
4. **Four overnight jobs until green:** distill → promote → embed → dream/spark. No wiki / ingest / KG / chaos / discovery on the metabolism path.
5. **Chaos is opt-in for chat** — `[chaos].enabled_in_chat = false` by default on the living config. Not on the daemon path.
6. **Product gate:** `gzmo status` answers “did last night work?” via `scheduler-runs/` + vault/honeypot counts — not Observatory as a second control plane.

## Consequences

- Lab recipes remain beat-gate fixtures, not the long-term production brain.
- `gzmo-scheduler` may still run lab parity jobs; living ops prefer `gzmo serve` metabolism.
- Graph/wiki/chaos return only after recall floors hold for consecutive nights.
