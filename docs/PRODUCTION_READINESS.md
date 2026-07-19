# Production readiness (index)

GZMO has **two** production bars. Do not mix them.

| Lane | Doc | Gate |
|------|-----|------|
| **Living stack** (CT101 overnight metabolism) | [LIVING_PRODUCTION_READINESS.md](LIVING_PRODUCTION_READINESS.md) | `bash scripts/living-readiness-gate.sh` |
| **Product MCP** (laptop Cursor/Pi attach) | [PRODUCT_PRODUCTION_READINESS.md](PRODUCT_PRODUCTION_READINESS.md) | `bash scripts/product-readiness-gate.sh` |

## Living stack (canonical)

```bash
bash scripts/living-readiness-gate.sh
# GREEN → data-next/living-readiness/latest.json
```

Owner: **CT101** `gzmo-daemon` + `/opt/gzmo/`. Workstation is operator/lab only — see [CT101_BOUNDARY.md](CT101_BOUNDARY.md).

## Product MCP

```bash
bash scripts/product-readiness-gate.sh
```

## Legacy workstation scripts

`scripts/verify-production.sh`, `scripts/p1-readiness-test.sh`, and `scripts/stack-closure-test.sh` still exist for historical workstation topology checks. Prefer the **living** and **product** gates above for current ops.
