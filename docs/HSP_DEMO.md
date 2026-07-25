# HSP metabolism demo (theater)

**Status:** Demable Unpark Wave 2.3 — **not** Brain Feed, **not** living GREEN  
**Front door:** `bash scripts/hsp-emit-demo.sh [--play]`  
**Sibling:** `~/github-clone/HSP` (optional `hsp ping`)

## What you hear

Metabolism artifacts (scheduler-runs, organ-trace, serendipity, promote pins, emit motif) → MIDI + WAV under `data-next/hsp-metabolism/`.

```bash
bash scripts/hsp-emit-demo.sh --play
bash scripts/hsp-emit-demo.sh --motif spark_flare --intensity 0.85 --play
bash scripts/hsp-emit-check.sh
```

`--play` preflights PipeWire default-sink volume (bumps if below 0.15) and prefers `pw-play`/`paplay`/`aplay -D default`. If you hear nothing, check `wpctl get-volume @DEFAULT_AUDIO_SINK@` before blaming WAV format.

## Motifs

| Motif | Feel |
|-------|------|
| `distill_tick` | Default night pulse |
| `spark_flare` | Brighter / higher |
| `dream_deep` | Longer / lower |
| `promote_pin` | Craft handoff echo |
| `serendipity` | Weekly apply color |

## Hard rules

1. Never block overnight metabolism on MIDI/WAV.  
2. Never claim Brain Feed strengthen from HSP.  
3. Living GREEN gates ignore this path.

## Artifacts

- `data-next/hsp-emit/latest-event.json` — motif emit  
- `data-next/hsp-metabolism/latest.{mid,wav,json,md}` — sonify output  
- `data-next/hsp-emit/emit-contract.md` — contract for sibling HSP  

See [STACK_OPPORTUNITY_MAP.md](./STACK_OPPORTUNITY_MAP.md) s2 · [UNPARK_ROADMAP.md](./UNPARK_ROADMAP.md) Wave 2.
