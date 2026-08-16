---
name: living-attach
description: Safe GZMO living MCP attach for external agents — prove vault, emit gzmo-living stanza, never dual-write or lab-allow.
triggers:
  - living attach
  - gzmo-living
  - hermes gzmo
  - external living mcp
requires_evidence: true
---

# Living attach (external agents)

## DO

1. `bash scripts/living-attach-check.sh` — must exit 0 (vault under living path, facts ≥100, dual-writer false).
2. `bash scripts/emit-living-mcp-fragment.sh --format hermes` — paste **`gzmo-living`** only.
3. Call living tools (`gzmo_memory_status` / search). Prefer nutrient / Brain Feed / airgap USP over ecosystem tourism.
4. OpenClaw: `bash scripts/install-openclaw-living-attach.sh` then search via MCP; write with `bash scripts/openclaw-takeaway.sh '…'` (never Qdrant upsert).

## NEVER

- `GZMO_PRODUCT=1` or `GZMO_ALLOW_LAB_VAULT=1` while claiming living
- Label living bridge as `gzmo-memory`
- Hand-rolled SSH MCP without `GZMO_CONFIG=/opt/gzmo/gzmo.toml`
- Enable workstation `gzmo-serve` / second overnight writer
- Treat `"connection closed: initialized request"` as attach success
- Rewrite CT101 from the attach path

## Docs

`docs/EXTERNAL_LIVING_ATTACH.md` · examples under `docs/examples/`
