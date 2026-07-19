# Product production readiness (Memory MCP)

**Audience:** Shipping the laptop Memory MCP product (not CT101 living stack)  
**Gate command:** `bash scripts/product-readiness-gate.sh`  
**GREEN:** exit `0` + `data-next/product-readiness/latest.json` → `"verdict": "GREEN"`

Living-stack readiness remains [PRODUCTION_READINESS.md](PRODUCTION_READINESS.md) / CT101. This doc is the **prime product** gate.

## Definition of GREEN

A stranger on Ubuntu can:

1. Install (`install-gzmo.sh` / `gzmo init`)
2. Attach Cursor/Pi MCP to `~/.gzmo` (not CT101 / not `data-next`)
3. Call `gzmo_memory_status` / `gzmo_memory_search`
4. Optionally metabolize a takeaway when Prime `:8000` (or any OpenAI-compatible engine) is up

Without requiring Redis, Qdrant, Neo4j, or overnight `gzmo serve`.

## Gate checks

| Check | Meaning |
|-------|---------|
| `verify-product-mcp` | Cold init → status/search → MCP tools + ops gate |
| `mcp-attach` | Cursor/Pi `gzmo-memory` → `~/.gzmo` + `GZMO_PRODUCT=1` |
| `product-config-hygiene` | No LAN/CT101 hosts in product toml |
| `product-engine` | `[engine].url` reachable (HOLD if down — metabolize optional) |
| `refresh-engine` | `gzmo init --refresh-engine` non-destructive scan |
| `product-hello` | Attach + first-fact loop (HOLD if no engine) |
| `prefer-prime-tests` | Unit: scanner prefers `:8000` |
| `ct101-living-owner` | Optional; `PRODUCT_GATE_REQUIRE_CT101=1` to require |
| `release-freshness` | Soft: tip within `RELEASE_FRESH_MAX` (default 5) of latest `v*` tag |

**FAIL** fails the gate. **HOLD** does not (optional engine / CT101 / stale release tag).  
After Keep lands on `main`, cut a new `v*` tag so stranger `install-gzmo.sh` ships tip features.

## Operator commands

```bash
# Co-primary A+C
bash scripts/production-readiness-gate.sh
# → data-next/production-readiness/latest.{json,md}

# Full product gate
bash scripts/product-readiness-gate.sh
# → data-next/product-readiness/latest.{json,md}

# Point existing ~/.gzmo at Prime without wiping vault
gzmo init --refresh-engine

# Rewire Cursor/Pi binary to ~/.local/bin/gzmo
MCP_ATTACH_FIX=1 bash scripts/mcp-attach-check.sh

# Stranger feel
bash scripts/product-hello-memory.sh
```

## Related

- [PRODUCT_MCP.md](PRODUCT_MCP.md) — install + tools
- [SPINE_FOCUS.md](SPINE_FOCUS.md) — Keep / Park doctrine
- [CT101_BOUNDARY.md](CT101_BOUNDARY.md) — living vault owner (separate lane)
