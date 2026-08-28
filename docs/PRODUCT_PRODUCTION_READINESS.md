# Client-attach smoke (`~/.gzmo`)

**Audience:** Regression for the historical `install-gzmo.sh` / `gzmo-memory` path  
**Gate command:** `bash scripts/product-readiness-gate.sh`  
**GREEN:** exit `0` + `data-next/product-readiness/latest.json` → `"verdict": "GREEN"`

**Not the product bar.** GZMO GREEN is [KEEP_QUALITY.md](KEEP_QUALITY.md) / [LIVING_PRODUCTION_READINESS.md](LIVING_PRODUCTION_READINESS.md) on the living host ([ADR-0007](./adr/ADR-0007-one-product-living.md)). This gate only proves the incomplete `~/.gzmo` installer still attaches.

## What GREEN means here

A machine can still:

1. Run `install-gzmo.sh` / `gzmo init`
2. Attach Cursor/Pi MCP to `~/.gzmo` (not CT101 / not `data-next`)
3. Call `gzmo_memory_status` / `gzmo_memory_search`
4. Optionally metabolize a takeaway when Prime `:8000` is up

That is **client scratch**, not a complete Keep. Passing this without sidecars/overnight does not ship GZMO.

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
# Product bar (living host)
bash scripts/keep-quality-gate.sh
bash scripts/living-readiness-gate.sh

# This page's smoke only
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

- [PRODUCT_MCP.md](PRODUCT_MCP.md) — historical installer (not a product)
- [ADR-0007-one-product-living.md](./adr/ADR-0007-one-product-living.md) — one SKU
- [SPINE_FOCUS.md](SPINE_FOCUS.md) — living Keep
- [CT101_BOUNDARY.md](./ops/CT101_BOUNDARY.md) — living vault owner
