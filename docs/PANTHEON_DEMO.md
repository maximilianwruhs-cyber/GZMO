# Pantheon ritual demo (theater)

**Status:** Demable Unpark Wave 2.1 — **not** Brain Feed, **not** living GREEN  
**Front door:** `bash scripts/pantheon-ritual-demo.sh`  
**Skills map:** [PANTHEON_SKILLS.md](./PANTHEON_SKILLS.md) · living boundary: [CHAOS_LIVING_VS_RITUAL.md](./CHAOS_LIVING_VS_RITUAL.md)

## What you feel

One-shot ritual skills (`dice` / `card` / `story`) via `gzmo chaos skill` — Wild Magic cascade, card forge, story brief. Feedback queues for chat/TUI drain. **Never** starts PulseLoop or the living daemon.

```bash
bash scripts/pantheon-ritual-demo.sh
bash scripts/pantheon-ritual-check.sh
bash scripts/pi/chaos_skill.sh dice d20 --json
bash scripts/pi/chaos_skill.sh card --json
bash scripts/pi/chaos_skill.sh story --json
```

Use `scripts/pi/chaos_skill.sh` (prefers the temp-bench release binary). A stale `target/release/gzmo` may open interactive chat instead of the one-shot skill path — rebuild or point `GZMO_BIN` / `CARGO_TARGET_DIR` at a binary that knows `chaos skill`.

## Skills

| Skill | Feel |
|-------|------|
| `dice` | Corpus narrative + nested cascade (may land on define/poem/…) |
| `card` | Card forge (legendary pack path) |
| `story` | CCL-aware story brief |

## Hard rules

1. Never wire daemon `dice_loop` fire into overnight living.  
2. Never claim Brain Feed / vault KPI credit from pantheon rolls.  
3. Slice C.1 pedagogy oscillator stays lab-only.  
4. Do not invent ghost `DICE_MASTER_*` files.

## Artifacts

- `data-next/pantheon-ritual/demo.json` — thin-skill inventory + re-land flags  
- `data-next/pantheon-ritual/felt-latest.{json,md}` — last dice/card/story felt sample  
- `data-next/pantheon-ritual/latest.json` — ritual check verdict  
- `data-next/pantheon-ritual/RELAND_CHECKLIST.md` — feat re-land hold list  

See [UNPARK_ROADMAP.md](./UNPARK_ROADMAP.md) Wave 2 · research archive under [research/pantheon/](./research/pantheon/).
