# Airgap living — one box, full Keep

**Status:** USP path (2026-07-20)  
**Doctrine:** [ADR-0004-airgap-living-usp.md](./ADR-0004-airgap-living-usp.md) · [SPINE_FOCUS.md](./SPINE_FOCUS.md)  
**Compose pin:** [LIVING_APPLIANCE.md](./LIVING_APPLIANCE.md) · `deploy/living-appliance/`  
**MCP:** [MCP_LOCAL_ATTACH.md](./MCP_LOCAL_ATTACH.md)

## Goal

Pull the ethernet. One machine still:

1. Extracts / verifies / promotes into vault → honeypot  
2. Runs overnight dream / spark / distill / immune / night lymph  
3. Serves agents via **local** `gzmo mcp-serve` (stdio or localhost subprocess)

No OpenRouter. No public webserver. No second overnight writer on a laptop.

## Single-box topology

```text
┌──────────────────── one airgapped box ────────────────────┐
│  Prime/llama.cpp :8000     embeddings :8081 (or local)      │
│  Redis :6379  Qdrant :6333  Neo4j :7687                     │
│  gzmo-daemon  →  data/vault.db + honeypot                   │
│  gzmo mcp-serve (stdio) ← Cursor / Pi on same box           │
└─────────────────────────────────────────────────────────────┘
```

CT101 (`/opt/gzmo/` + `/opt/database-cluster`) is the **reference** deployment. The USP is any box that matches this shape.

## Bring-up (hero path)

```bash
# From a clone on the target machine:
bash scripts/install-living-airgap.sh

# Or step-by-step:
bash scripts/living-appliance-up.sh          # Redis/Qdrant/Neo4j compose pin
# Install/build gzmo; point GZMO_CONFIG at living toml (see below)
# Enable gzmo-daemon (sole overnight writer on THIS box)
# Wire local MCP: stdio → gzmo mcp-serve with that config
bash scripts/keep-quality-gate.sh            # when host is the living brain
```

Config sketch: [`config/living-appliance.gzmo.toml.example`](../config/living-appliance.gzmo.toml.example) — enable `[redis]` / `[qdrant]`, local engine URLs, Neo4j MCP fragment. Secrets via `.env` only.

Local engines (typical):

| Service | Default | Role |
|---------|---------|------|
| Prime / llama.cpp | `http://127.0.0.1:8000` | extract / verify / dream / spark |
| Embeddings | `http://127.0.0.1:8081` | honeypot / Qdrant vectors |
| Redis | `redis://127.0.0.1:6379` | distill queue + scratch |
| Qdrant | `http://127.0.0.1:6333` | vector recall |
| Neo4j | `bolt://127.0.0.1:7687` | graph MCP |

See [PORTS.md](./PORTS.md), [CT101_DEPLOY.md](./CT101_DEPLOY.md) for production scars on the reference host.

## Honest degraded modes

Never pretend overnight living is up when it is not.

| Missing | Honest mode | Overnight writer |
|---------|-------------|------------------|
| Redis / Qdrant / Neo4j | **Lite attach** only (`~/.gzmo` or vault without sidecars) | **No** |
| Local LLM down | Pause distill/dream; surface health FAIL/WARN | Daemon may idle; do not fail open to cloud unless operator opt-in |
| Embeddings down | FTS-only recall; hold Qdrant sync | Allowed with WARN; do not claim full honeypot vector path |
| Second host already writing overnight | **Refuse** to enable daemon writers here | ADR-0003 |

Degrade messaging must say **lite / incomplete**, never “living GREEN.”

## Airgap honesty checklist

- [ ] `active_mode` / engine URLs point at **localhost** (or on-box LAN you own), not OpenRouter  
- [ ] Cloud LLM keys absent or unused for core metabolism  
- [ ] Sidecars bound to `127.0.0.1` (or host firewall denies WAN)  
- [ ] MCP fragment is stdio / local binary — not a public HTTP endpoint  
- [ ] Exactly one overnight writer on this vault  

## Lite vs living

| | Lite | Living |
|--|------|--------|
| Installer hero? | No — bootstrap fallback | **Yes** |
| `install-gzmo.sh` | Day-zero MCP | Points here for full USP |
| Quality gate | `product-readiness-gate.sh` | **`keep-quality-gate.sh`** |

## Verify

```bash
bash scripts/living-appliance-gate.sh
bash scripts/keep-quality-gate.sh
# Optional soak (N nights):
bash scripts/keep-quality-soak.sh
```
