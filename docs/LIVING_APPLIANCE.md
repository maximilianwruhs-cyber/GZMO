# Living appliance (goal C)

**Status:** Keep goal — compose pin shipped (2026-07-19)  
**Paired with:** [PRODUCT_MCP.md](./PRODUCT_MCP.md) (goal A)  
**Doctrine:** [SPINE_FOCUS.md](./SPINE_FOCUS.md) · [research/CT101_STACK_FUTURE_2026-07.md](./research/CT101_STACK_FUTURE_2026-07.md)

## What this is

A **preconfigured one-writer stack**:

```text
gzmo-daemon + SQLite vault/honeypot + Redis + Qdrant + Neo4j
```

Today that runs on **CT101** (`/opt/gzmo/` + `/opt/database-cluster`). Goal C makes the sidecar shape **reproducible in-repo**.

## In-repo pin

| Path | Role |
|------|------|
| [`deploy/living-appliance/docker-compose.yml`](../deploy/living-appliance/docker-compose.yml) | Redis `:6379`, Qdrant `:6333`/`:6334`, Neo4j `:7474`/`:7687` |
| [`deploy/living-appliance/.env.example`](../deploy/living-appliance/.env.example) | `NEO4J_AUTH=neo4j/…` (copy to `.env`, gitignored) |
| [`config/living-appliance.gzmo.toml.example`](../config/living-appliance.gzmo.toml.example) | Daemon `[redis]` / `[qdrant]` / Neo4j MCP fragment |
| [`scripts/living-appliance-gate.sh`](../scripts/living-appliance-gate.sh) | Pin validity gate → `data-next/living-appliance/` |

```bash
# One-shot sidecar bring-up + gate
bash scripts/living-appliance-up.sh

# Or manually:
cd deploy/living-appliance
cp .env.example .env   # set NEO4J_AUTH
docker compose up -d
bash ../../scripts/living-appliance-gate.sh
```

This compose starts **sidecars only**. Pair with `gzmo-daemon` + `/opt/gzmo/gzmo.toml` (see [CT101_DEPLOY.md](./CT101_DEPLOY.md)).

## Labeled MCP attach (A vs C)

| Server name | Goal | Install |
|-------------|------|---------|
| `gzmo-memory` | **A** product `~/.gzmo` | `scripts/install-product-mcp.sh` / `install-gzmo.sh` |
| `gzmo-living` | **C** CT101 vault via SSH | `scripts/install-shared-mcp.sh` |

`install-shared-mcp.sh` migrates a mislabeled living entry off `gzmo-memory` → `gzmo-living` and restores product `gzmo-memory` from `~/.gzmo/mcp.json` when present.

## What this is not

| Not | Why |
|-----|-----|
| Stranger laptop product | That is **A** — `~/.gzmo`, sidecars off |
| Pi-first UX | Optional glass only |
| Two overnight writers | [ADR-0003](./ADR-0003-one-instance-metabolism.md) |
| Secrets in git | `NEO4J_AUTH` only via `.env` |

## Ports

See [PORTS.md](./PORTS.md). Qdrant image is pinned (`v1.13.2`) for reproducible bring-up; CT101 may still run `qdrant/qdrant:latest` until operators migrate.

## Ops scar (CT101)

Live `/opt/database-cluster/docker-compose.yml` historically embedded Neo4j auth in plaintext. Prefer migrating that host to this pin + `.env`, and **rotate** any password that ever lived in compose or agent homes ([AGENT_HOME_SECRETS.md](./AGENT_HOME_SECRETS.md)).

## Verify

```bash
bash scripts/living-appliance-gate.sh
bash scripts/living-readiness-gate.sh   # includes soft appliance-pin row
```
