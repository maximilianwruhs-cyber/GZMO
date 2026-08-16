# ADR-0006 — One vault owner, two socket clients

**Status:** Accepted (2026-08-16) — implemented in-tree  
**Related:** [ADR-0003](./ADR-0003-one-instance-metabolism.md), [ADR-0004](./ADR-0004-airgap-living-usp.md), [ADR-0005](./ADR-0005-flywheel-over-frozen-topology.md)

## Context

Daemon, CLI, and `mcp-serve` were peer consumers of `gzmo-core` / `PlatformMemory`. Dual-writer was prevented by doctrine (`living-host-mutex.sh`, env checklists), not by a process boundary. Mentor already had a Unix socket; it was a one-off.

## Decision

1. **`gzmo serve` and `gzmo daemon` are owners.** They take an exclusive flock on `{vault_db}.write.lock` and listen on a Unix socket (default `{vault_db.parent()}/gzmo.sock`, `0600`).
2. **CLI (`gzmo memory *`) and MCP are clients.** They prefer the socket when it is live. `--offline` or `GZMO_CONTROL_PLANE=0` forces in-process `PlatformMemory` (lite / telescope).
3. **No HTTP.** No MCP-as-control-plane. Protocol is one NDJSON request per connection (`ping`, `memory.search|recall|status|turn_start|chain|profile`).
4. **No auto-fallback that opens a living vault when the socket is down and the profile is living.** That hard-fail is a later graft. This change still allows in-process open so CT101/`~/.gzmo` keep working during rollout.
5. **Host mutex stays host placement** (`ct101` vs `workstation`). Process ownership is the flock.

## Consequences

- A second `serve`/`daemon` on the same vault dies at lock acquire.
- Cursor MCP on a box where the owner is up no longer opens a second long-lived vault handle.
- Local rebuild/proof is `cargo test -p gzmo-core control_plane` — not `gzmo serve` against the living vault on the telescope.
