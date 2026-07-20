# Brain Feed — satellites that nourish the living vault

**Status:** Active Unpark focus (2026-07-20)  
**USP:** [ADR-0004-airgap-living-usp.md](./ADR-0004-airgap-living-usp.md)  
**Gate:** `bash scripts/brain-feed-check.sh` → `data-next/brain-feed/`  
**Quality bar (still required):** [KEEP_QUALITY.md](./KEEP_QUALITY.md)

## Lock

Only these Unpark satellites claim to **improve** the living vault/honeypot. Theater (HSP, pantheon, Observatory glass, €/night display) is demoted — demable, not “next strengthen.”

| Tier | Satellite | How the brain profits |
|------|-----------|------------------------|
| **P0** | herdr + takeaway ritual | Pane close → `[TAKEAWAY]` → distill → honeypot |
| **P0** | tinyFolder | Drop → inbox → ingest / takeaway enqueue toward **living** queue |
| **P0** | Memory MCP Felt Use | Real search → `recall_count` → ripen can fire |
| **P0** | Serendipity promote-back | Spark digest → verified links → takeaway/promote (dry-run default) |
| **P0** | Dream compaction | Hygiene — less DREAMS noise (soft; off GREEN math) |
| **P1** | Calibration → living pin | Fused decode suggestion → **human** merges into living toml |
| **P1** | Arena → human promote | Champion suggestion only — smarter overnight model after human pin |
| **P1b** | IpW / Forge | Same pattern after Arena suggestion is boring |

## Hard rules

1. **One overnight writer** ([ADR-0003](./ADR-0003-one-instance-metabolism.md)) — Brain Feed never starts `gzmo serve` / daemon on the workstation while CT101 (or another living host) owns metabolism.
2. **No auto engine swap** — Arena / Forge / IpW / calibration emit **suggestions** only.
3. **Living vault target** — nutrient paths aim at the living host (`/opt/gzmo` today), not `~/.gzmo` lite and not silent `data-next` overnight writers.
4. **keep-quality stays the USP bar** — Brain Feed GREEN does not replace soak readiness.

## Operator loops

### Takeaway / herdr (P0)

```bash
# Close ritual enqueues distill (no --now on workstation while CT101 lives)
bash scripts/herdr-metabolism-link.sh
# Proof:
bash scripts/herdr-metabolism-check.sh
bash scripts/ct101-takeaway-recall.sh   # living same-sitting HIT
```

See [HERDR_METABOLISM.md](./HERDR_METABOLISM.md).

### tinyFolder → living (P0)

```bash
bash scripts/tinyfolder-drop.sh --demo --living
bash scripts/tinyfolder-check.sh
# living-enqueue.json advises CT101 ingest/takeaway; never dual-writer
```

### Serendipity promote-back (P0)

```bash
bash scripts/serendipity-digest.sh
bash scripts/serendipity-promote.sh          # dry-run default
# SERENDIPITY_PROMOTE_APPLY=1 bash scripts/serendipity-promote.sh   # human-gated
```

### Intelligence promote (P1) — no auto

```bash
bash scripts/brain-intel-promote.sh
# → data-next/brain-intel/latest.json
# Review calibration suggestion + Arena champion-suggestion.toml; merge by hand on living host
```

### Dream compact (P0 hygiene)

```bash
bash scripts/dream-compact-lab.sh   # dry-run / lab
# Living soft window: operator runs compact on living host only; never flips keep-quality GREEN math
```

## Verify

```bash
bash scripts/brain-feed-check.sh
# → data-next/brain-feed/latest.{json,md}
```

## Out / demoted (not Brain Feed)

HSP sonification · pantheon / discovery theater · Observatory public mind · €/night display · AOS CE packaging · marketplace read-only · Cognis / ZPD / escape-loop (never-as-brain).
