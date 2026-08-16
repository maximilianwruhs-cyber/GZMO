# MCP local attach (brand path)

**Status:** Brand attach contract (2026-07-20)  
**Doctrine:** [ADR-0004](./ADR-0004-airgap-living-usp.md) · [ADR-0007](./ADR-0007-one-product-living.md)  
**Deep dive:** [ct101-systems/70-mcp-layer/mcp-serve.md](./ct101-systems/70-mcp-layer/mcp-serve.md)

## Contract

Agents (Cursor / Pi / CLI) attach to GZMO memory via **stdio MCP** — the client spawns `gzmo mcp-serve` as a subprocess. Optional bind is **localhost only** if a future transport wraps stdio; **public HTTP/SSE MCP is not the product SKU** (lab/GZMO-next only).

```text
Cursor / Pi  →  stdio  →  gzmo mcp-serve  →  living vault (`gzmo-living`)
```

## Labels

| Server label | Role | Config | Installer |
|--------------|---------|--------|-----------|
| `gzmo-living` | **Brand** — living writer | living `GZMO_CONFIG` | On-box: local `mcp-serve`; ops: `install-shared-mcp.sh` → SSH bridge |
| `gzmo-memory` | **Legacy** scratch | `~/.gzmo/gzmo.toml` | `scripts/install-gzmo.sh` — not a product ([ADR-0007](./ADR-0007-one-product-living.md)) |

Living brand path on the airgap box:

```json
{
  "mcpServers": {
    "gzmo-living": {
      "command": "/usr/local/bin/gzmo",
      "args": ["mcp-serve"],
      "env": {
        "GZMO_CONFIG": "/opt/gzmo/gzmo.toml"
      }
    }
  }
}
```

Adjust binary/config paths to the host. Do **not** set `GZMO_PRODUCT=1` for living.

## What is out of brand scope

| Pattern | Verdict |
|---------|---------|
| Public multi-tenant MCP URL | **Out** |
| Stranger product pointing at CT101 over the internet | **Out** |
| Native MCP-over-HTTP as default install | **Out** (lab only) |
| SSH stdio bridge to a box you own (`pi-gzmo-mcp-serve.sh`) | **Ops topology** — allowed for operators; not the airgap USP story |

## Hardening checklist

- [ ] Fragment uses absolute path to a trusted `gzmo` binary  
- [ ] `GZMO_CONFIG` points at the living vault (legacy `~/.gzmo` is not the product)  
- [ ] Living overnight writer is on the **same** machine as the vault (or you accept ops SSH and still keep one writer)  
- [ ] No WAN-exposed MCP port in compose or firewall  
- [ ] Secrets stay in `.env` / process env — not in committed `mcp.json`  

## Verify

```bash
# Brand: living label, not gzmo-memory
bash scripts/living-mcp-attach-check.sh

# Fail-closed living vault proof (external agents / Hermes)
bash scripts/living-attach-check.sh
bash scripts/emit-living-mcp-fragment.sh --format hermes

# On the living box itself (preferred USP):
GZMO_CONFIG=/path/to/living.toml gzmo mcp-serve   # client-spawned; do not leave listening on 0.0.0.0
```

## Related

- [EXTERNAL_LIVING_ATTACH.md](./EXTERNAL_LIVING_ATTACH.md) — agent DO/NEVER + emit/check kit  
- [PRODUCT_MCP.md](./PRODUCT_MCP.md) — historical `~/.gzmo` installer (not a product)  
- [AIRGAP_LIVING.md](./AIRGAP_LIVING.md) — living hero path  
- [PI_GZMO_MEMORY_INTEGRATION.md](./PI_GZMO_MEMORY_INTEGRATION.md) — ops SSH living attach  
