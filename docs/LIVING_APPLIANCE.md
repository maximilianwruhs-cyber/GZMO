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
| [`scripts/ct101-living-appliance-smoke.sh`](../scripts/ct101-living-appliance-smoke.sh) | CT101 protocol smoke → `data-next/living-appliance-smoke/` |
| [`scripts/living-appliance-smoke.sh`](../scripts/living-appliance-smoke.sh) | Local pin protocol smoke (lab / pin up) |
| [`scripts/living-appliance-health-smoke.sh`](../scripts/living-appliance-health-smoke.sh) | Daemon health via lab `GZMO_CONFIG` (never `~/.gzmo`) → `data-next/living-appliance-health/` |
| [`scripts/ct101-living-appliance-pin-check.sh`](../scripts/ct101-living-appliance-pin-check.sh) | Staged pin vs live `/opt/database-cluster` shape → `data-next/living-appliance-pin-ct101/` |
| [`scripts/ct101-promote-living-appliance-auth.sh`](../scripts/ct101-promote-living-appliance-auth.sh) | Move live inline `NEO4J_AUTH` → `.env` + pin Qdrant tag |

```bash
# One-shot sidecar bring-up + gate + smoke
bash scripts/living-appliance-up.sh

# Or manually:
cd deploy/living-appliance
cp .env.example .env   # set NEO4J_AUTH
docker compose up -d
bash ../../scripts/living-appliance-gate.sh
bash ../../scripts/living-appliance-smoke.sh
bash ../../scripts/living-appliance-health-smoke.sh
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
| Workstation Neo4j as living SoT | Throwaway until pin/product finalize (see Auth below) |

## Auth (Neo4j)

Two shapes, one secret:

| Surface | Variable | Format |
|---------|----------|--------|
| Compose / Neo4j image | `NEO4J_AUTH` | `neo4j/<password>` in pin `.env` (gitignored) |
| GZMO daemon / MCP | `NEO4J_PASSWORD` | password only (`/opt/gzmo/.env` on CT101) |

**Operator lock (2026-07-19):** workstation Neo4j (`~/database-cluster` or any local `sidecar-neo4j`) is **throwaway**. Do not copy its password into the in-repo pin. Do not treat bolt-open on the laptop as goal-C auth proof. Auth SoT is CT101: `/opt/database-cluster/.env` (`NEO4J_AUTH`) + `/opt/gzmo/.env` (`NEO4J_PASSWORD`).

Living readiness protocol smoke targets **CT101** (`scripts/ct101-living-appliance-smoke.sh`), not the workstation throwaway stack.

## Ports

See [PORTS.md](./PORTS.md). Qdrant image is pinned to the living host version (`v1.18.1` as of 2026-07-19) for reproducible bring-up without downgrade.

## Ops scar (CT101)

Live `/opt/database-cluster/docker-compose.yml` historically embedded Neo4j auth in plaintext. Migrated via `scripts/ct101-promote-living-appliance-auth.sh` → `.env` + `${NEO4J_AUTH}`. Prefer rotating any password that ever lived in compose or agent homes ([AGENT_HOME_SECRETS.md](./AGENT_HOME_SECRETS.md)).

## Verify / sync

```bash
bash scripts/living-appliance-gate.sh
bash scripts/ct101-living-appliance-smoke.sh  # CT101 Redis/Qdrant/Neo4j protocol
bash scripts/living-appliance-health-smoke.sh # lab GZMO_CONFIG → redis/qdrant/neo4j
bash scripts/living-mcp-attach-check.sh
bash scripts/ct101-sync-living-appliance.sh   # stage pin under /opt/gzmo/current/…
bash scripts/ct101-living-appliance-pin-check.sh  # staged vs live shape
bash scripts/ct101-promote-living-appliance-auth.sh  # one-shot: live compose → .env auth
bash scripts/living-readiness-gate.sh
```
