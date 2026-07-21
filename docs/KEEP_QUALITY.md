# Keep quality gate

**Status:** Continuous living quality bar (2026-07-20)  
**USP:** [ADR-0004-airgap-living-usp.md](./ADR-0004-airgap-living-usp.md)  
**Script:** `bash scripts/keep-quality-gate.sh` → `data-next/keep-quality/`  
**Soak:** `bash scripts/keep-quality-soak.sh`

## What it proves

Not “binary installs.” That the **living box** still compounds honeypot-quality memory under airgap-honest ops:

| Pillar | Check |
|--------|--------|
| Ops | Living readiness GREEN (or local living smoke) |
| Felt recall | Faithfulness living floor |
| Felt Use | Nonzero `recall_count` share on latest honeypot |
| Felt Use depth | Soft: `recall≥3` count/share for honest ripen (`felt-use-depth.sh`) — thin = HOLD, not RED |
| Spark | Refractory last-N unique anchors ≫ 1 |
| Immune | Plan artifact present; closed-class candidate count reported |
| Ripen | Honest status (Ready / starved_recall — never empty-core lie) |
| Lymph | `night-lymph/latest.json` present |
| Attach | Living MCP label / local attach check |
| Airgap honesty | Local engine preferred; cloud not required for core path |

## Run

```bash
# Against CT101 reference (default SSH host ct101):
bash scripts/keep-quality-gate.sh
bash scripts/felt-use-depth.sh          # depth census alone (recall≥1 / ≥3 + ripen dual)

# Skip heavy takeaway during soak loops:
LIVING_GATE_SKIP_TAKEAWAY=1 bash scripts/keep-quality-gate.sh

# Record a soak night (appends history; exits 0 only if GREEN):
bash scripts/keep-quality-soak.sh
```

Artifacts:

- `data-next/keep-quality/latest.json` / `latest.md` / `gate.log`
- `data-next/keep-quality/soak-log.jsonl` (from soak script)

## Unpark gate

Do **not** expand Unpark Wave 1 as brand work until soak history shows **honest** GREEN nights (default **3** — `KEEP_QUALITY_SOAK_NIGHTS`) with ≥**18h** between counted samples (`KEEP_QUALITY_SOAK_MIN_HOURS`). Same-hour GREEN streaks → HOLD (`soak_spacing_hold`), not `soak_ready_unpark_ok`. Surfaces are local MCP clients of this living brain.

```bash
bash scripts/keep-quality-soak.sh --summary
# → honest_nights / min_hours / advice
```

