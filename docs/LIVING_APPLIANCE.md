# Living appliance (goal C)

**Status:** Keep goal (operator lock 2026-07-19)  
**Paired with:** [PRODUCT_MCP.md](./PRODUCT_MCP.md) (goal A)  
**Doctrine:** [SPINE_FOCUS.md](./SPINE_FOCUS.md) · [research/CT101_STACK_FUTURE_2026-07.md](./research/CT101_STACK_FUTURE_2026-07.md)

## What this is

A **preconfigured one-writer stack**:

```text
gzmo-daemon + SQLite vault/honeypot + Redis + Qdrant + Neo4j
```

Today that runs on **CT101** (`/opt/gzmo/` + Docker sidecars). Goal C is to make that shape **demable and reproducible** (in-repo compose + gate), not a hand-assembled LXC folklore.

## What this is not

| Not | Why |
|-----|-----|
| Stranger laptop product | That is **A** — `~/.gzmo`, sidecars off |
| Pi-first UX | Optional glass only |
| Two overnight writers | [ADR-0003](./ADR-0003-one-instance-metabolism.md) |
| “GZMO already ships compose in-repo” | **Not yet** — C’s next ship work |

## Ports (locked)

See [PORTS.md](./PORTS.md): Redis `:6379`, Qdrant `:6333`, Neo4j `:7687`, plus cognition/embed off-box as configured.

## Next ship shape

1. In-repo compose (or compose pin) for Redis/Qdrant/Neo4j matching CT101  
2. `scripts/living-appliance-gate.sh` (or extend living-readiness) proving sidecars + daemon health  
3. Labeled living MCP attach docs (`gzmo-living` vs product `gzmo-memory`)  
4. Restore/runbook: [CT101_DEPLOY.md](./CT101_DEPLOY.md), [CT101_PATH_AUTHORITY.md](./CT101_PATH_AUTHORITY.md)

## Verify today

```bash
bash scripts/living-readiness-gate.sh
# → data-next/living-readiness/latest.json
```
