# Production readiness (index)

GZMO has **one** product bar: living Keep health ([ADR-0007](./adr/ADR-0007-one-product-living.md)). Client-attach scripts are smoke, not a second SKU.

| Lane | Doc | Gate |
|------|-----|------|
| **Living Keep** (the product) | [LIVING_PRODUCTION_READINESS.md](LIVING_PRODUCTION_READINESS.md) · [KEEP_QUALITY.md](KEEP_QUALITY.md) | `bash scripts/keep-quality-gate.sh` · `bash scripts/living-readiness-gate.sh` |
| **Client attach smoke** (not a product GREEN) | [PRODUCT_PRODUCTION_READINESS.md](PRODUCT_PRODUCTION_READINESS.md) | `bash scripts/product-readiness-gate.sh` |

## Living stack (canonical)

```bash
bash scripts/keep-quality-gate.sh
bash scripts/living-readiness-gate.sh
# GREEN → data-next/keep-quality/ and data-next/living-readiness/latest.json
```

Owner: the host that holds `living-host-mutex` (CT101 reference: `gzmo-daemon` + `/opt/gzmo/`). Telescope workstation does not run a second writer — see [CT101_BOUNDARY.md](./ops/CT101_BOUNDARY.md).

## Client attach smoke

```bash
bash scripts/product-readiness-gate.sh
```

Incomplete `~/.gzmo` passing this gate is **not** a complete GZMO.

## Legacy workstation scripts

`scripts/verify-production.sh`, `scripts/p1-readiness-test.sh`, and `scripts/stack-closure-test.sh` still exist for historical workstation topology checks. Prefer the **living** gates above for current ops.
