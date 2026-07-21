# Brain Feed — satellites that nourish the living vault

**Status:** Active Unpark focus (2026-07-20)  
**USP:** [ADR-0004-airgap-living-usp.md](./ADR-0004-airgap-living-usp.md)  
**Gate:** `bash scripts/brain-feed-check.sh` → `data-next/brain-feed/`  
**Quality bar (still required):** [KEEP_QUALITY.md](./KEEP_QUALITY.md)  
**What to build next:** [OPPORTUNITY_DISCOVERY.md](./OPPORTUNITY_DISCOVERY.md)

## Lock

Only these Unpark satellites claim to **improve** the living vault/honeypot. Theater (HSP, pantheon, Observatory glass, €/night display) is demoted — demable, not “next strengthen.”

| Tier | Satellite | How the brain profits |
|------|-----------|------------------------|
| **P0** | herdr + takeaway ritual | Pane close → `[TAKEAWAY]` → distill → honeypot |
| **P0** | tinyFolder | Drop → inbox → ingest / takeaway enqueue toward **living** queue |
| **P0** | Memory MCP Felt Use | Real search → `recall_count` → ripen can fire |
| **P0** | Felt Use depth floor | Census `recall≥3` share — ripen dual-gate honesty (HOLD if thin) |
| **P0** | Serendipity promote-back | Spark digest → verified links → takeaway/promote (dry-run default) |
| **P0** | Dream compaction | Hygiene — less DREAMS noise (soft; off GREEN math) |
| **P1** | Calibration → living pin | Fused decode suggestion → **human** merges into living toml |
| **P1** | Arena → human promote | Champion suggestion only — smarter overnight model after human pin |
| **P1b** | IpW / Forge | Same pattern after Arena suggestion is boring |

## Hard rules

1. **One overnight writer** ([ADR-0003](./ADR-0003-one-instance-metabolism.md) / [ADR-0005](./ADR-0005-flywheel-over-frozen-topology.md)) — Brain Feed never races a second host. If the workstation holds the living claim (`living-host-mutex.sh`), feed *that* host; otherwise do not start `gzmo serve` while CT101 owns metabolism.
2. **No auto engine swap** — Arena / Forge / IpW / calibration emit **suggestions** only.
3. **Living vault target** — nutrient paths aim at the living host (`/opt/gzmo` today), not `~/.gzmo` lite and not silent `data-next` overnight writers.
4. **keep-quality stays the USP bar** — Brain Feed GREEN does not replace soak readiness.

## Operator loops

### Takeaway / herdr (P0)

**Side-effect only** — takeaways piggyback on work you already paid for (Cursor credits / real sessions). Do **not** open a second agent chat whose only job is “feed the vault.”

| Do | Don't |
|----|-------|
| End a real coding/ops session with one `gzmo session close --takeaway` (or herdr close-ritual) aimed at the **living** host | Start a memory-gym Cursor session |
| Use PR template + optional git hook as *reminders* | Treat reminders as a second overnight writer |
| Leave `--now` alone while CT101 owns metabolism | Burn superior-model credits to practice takeaways |

```bash
# Close ritual enqueues distill (no --now on workstation while CT101 lives)
bash scripts/herdr-metabolism-link.sh
# Living proof (SSH session close on CT101 — pane-close contract without memory gym):
bash scripts/herdr-living-enqueue.sh
# Remind surfaces (PR template + optional local hook):
bash scripts/takeaway-side-effect-remind.sh
# bash scripts/takeaway-side-effect-remind.sh --install-hook
# Proof:
bash scripts/herdr-metabolism-check.sh
bash scripts/ct101-takeaway-recall.sh   # living same-sitting HIT
```

See [HERDR_METABOLISM.md](./HERDR_METABOLISM.md).

### Felt Use depth (P0)

Nonzero recall is necessary but shallow. Ripen dual-gate needs `recall≥3`. Measure only — never open a memory-gym chat to inflate counts.

```bash
bash scripts/felt-use-depth.sh
# → data-next/felt-use-depth/latest.{json,md}
# depth thin ⇒ HOLD advice; census fail ⇒ RED; brain-feed / keep-quality wire the row
```

### tinyFolder → living (P0)

```bash
# Dry-run artifact (no SSH apply):
bash scripts/tinyfolder-drop.sh --demo --living
# One-shot enqueue on living host (session close --takeaway, no --now):
bash scripts/tinyfolder-drop.sh --demo --living --apply-takeaway
# Or: TINYFOLDER_APPLY_TAKEAWAY=1 bash scripts/tinyfolder-drop.sh --living note.md
# Overnight organ (pending drops → living enqueue, no CLI):
bash scripts/tinyfolder-overnight.sh --dry-run
# On CT101: bash scripts/tinyfolder-overnight.sh --on-host
# Install daily timer (~02:45 UTC) on living host:
#   ssh ct101 'bash /opt/gzmo/current/scripts/install-tinyfolder-overnight-timer.sh'
bash scripts/tinyfolder-check.sh
# living-enqueue.json: proposed takeaways + applied[] when --apply-takeaway;
# refuses if workstation gzmo-serve is active (dual-writer)
```

### Serendipity promote-back (P0)

**Weekly cadence** (cron-friendly; never auto-applies):

```bash
bash scripts/serendipity-cadence.sh
# → data-next/serendipity/cadence-latest.{json,md} + cadence-log.jsonl
```

Checklist after a spark night:

1. `bash scripts/serendipity-cadence.sh` (digest + dry-run + reminder)  
2. Review `data-next/serendipity/promote-latest.json` (≤3 takeaways)  
3. If clear: `bash scripts/serendipity-apply-proof.sh --apply`  
   (or `SERENDIPITY_PROMOTE_APPLY=1 bash scripts/serendipity-promote.sh`)  
4. `bash scripts/brain-feed-check.sh` stays GREEN  

```bash
bash scripts/serendipity-digest.sh
bash scripts/serendipity-promote.sh          # dry-run only
bash scripts/serendipity-apply-proof.sh      # dry report
# bash scripts/serendipity-apply-proof.sh --apply   # human-gated ≤3
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

## After merge → living host (script-only)

When Brain Feed / keep-quality scripts land on `main` and the binary is unchanged:

```bash
bash scripts/ct101-brain-feed-sync.sh   # rsync + restore +x; never restarts gzmo-daemon
```

See [CT101_DEPLOY.md](./CT101_DEPLOY.md) §“Sync docs/scripts only”.

## Out / demoted (not Brain Feed)

HSP sonification · pantheon / discovery theater · Observatory public mind · €/night display · AOS CE packaging · marketplace read-only · Cognis / ZPD / escape-loop (never-as-brain).
