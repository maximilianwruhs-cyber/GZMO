# ARCH-DIR-001 — GZMO Sovereignty Constitution

Adapted from ARCH-DIR-001 v2.1 for the GZMO Rust stack. This document is normative for sovereign-mode operation.

## Principles

1. **Local-first** — Prime cognition defaults to `localhost:8000`; cloud is opt-in only.
2. **Data sovereignty** — Vault, honeypot, and episodic data stay on the homelab perimeter.
3. **Lean growth** — New dependencies require a Zero-Bloat Review; no silent bloat.
4. **Plaintext observability** — Logs, `gzmo health`, and Obolus ledger; no third-party telemetry SDKs.
5. **Energy bilanz** — All Prime consumption is measured; autonomous actions require ObolusGate approval.

## Trusted perimeter

- Workstation: Prime `:8000`, GZMO daemon, Pi agent
- LAN `192.168.31.0/24`: embeddings VM200, Qdrant/Neo4j/Redis on LXC101
- No Internet required for the core cognition path in sovereign mode

## Guardrails

| Area | GZMO approach | Forbidden in sovereign mode |
|------|---------------|----------------------------|
| Cognition | `engine.local` → Prime | `active_mode=cloud` as default |
| Memory | SQLite vault + honeypot | Cloud memory SaaS |
| Retrieval | Qdrant/Neo4j on LAN | Public vector DBs |
| Tools | MCP stdio, governed shell | `web_search` without opt-in |
| Observability | logs/, `gzmo health`, Obolus ledger | Phoning-home SDKs |
| Energy | ObolusGate + system bilanz | Autospawn/loops without bilanz |
| Dependencies | Workspace-pinned, reviewed | New crate without Zero-Bloat Review |

## Energy bilanz (Obolus) — binding

1. Prime is a finite resource; consumption is recorded in `data/Obolus/ledger.jsonl`.
2. Tier T2 autonomous paths (Kurator autospawn, discovery fixer, daemon dream/spark/dice) require ObolusGate allow.
3. Tier T0 operator paths (chat, manual approve) are measured and warned, never hard-denied.
4. Decisions use `E_total`, `ctx_%`; η is advisory until Phase C.

See [OBOLUS_GOVERNANCE.md](OBOLUS_GOVERNANCE.md) and [OBOLUS_EFFICIENCY.md](OBOLUS_EFFICIENCY.md).

## What we do not adopt from ARCH-DIR-001

- ≤5 dependencies (we baseline ~20 workspace deps)
- 10 MB binary cap (warn at ~80 MB)
- Mandatory Nix
- Go `cmd/internal/` layout (Rust workspace is normative)
- Removing Neo4j/Qdrant (homelab perimeter)

## Document chain

`ARCH-DIR-001.md` (root pointer) → this file → `MACHINE.md` → `INFRASTRUCTURE_OVERVIEW.md` → `ARCHITECTURE_GZMO_PLATFORM.md`

## Verification

```bash
./scripts/sovereignty-verify.sh
gzmo obolus status
```
