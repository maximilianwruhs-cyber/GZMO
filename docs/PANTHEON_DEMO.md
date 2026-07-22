# Pantheon ritual demo (theater)

**Status:** Demable Unpark Wave 2.1 — **not** Brain Feed, **not** living GREEN  
**Front door:** `bash scripts/pantheon-ritual-demo.sh`  
**Skills map:** [PANTHEON_SKILLS.md](./PANTHEON_SKILLS.md) · living boundary: [CHAOS_LIVING_VS_RITUAL.md](./CHAOS_LIVING_VS_RITUAL.md)

## What you feel

One-shot ritual skills (`dice` / `card` / `story`) via `gzmo chaos skill`. Feedback queues for chat/TUI drain. **Never** starts PulseLoop or the living daemon.

```bash
bash scripts/pantheon-ritual-demo.sh
# → data-next/pantheon-ritual/felt-latest.md   (human-readable)
# → bash scripts/pantheon-ritual-check.sh      (expect GREEN)

bash scripts/pi/chaos_skill.sh dice d20 --json
bash scripts/pi/chaos_skill.sh card --json
bash scripts/pi/chaos_skill.sh story --json
```

`scripts/pi/chaos_skill.sh` **probes** binaries and refuses a stale `gzmo` that opens interactive chat instead of the one-shot path. Rebuild if it complains:

```bash
CARGO_TARGET_DIR=$HOME/github-clone/temp-bench/target cargo build -p gzmo-cli --release
```

## Skills

| Skill | Feel |
|-------|------|
| `dice` | Chaos roll + event text; often **Wild Magic** cascading into another skill (`define` / `poem` / …). `--json` reports `skill: "dice"` with `cascade_skill` / `cascade` for the child. |
| `card` | Card forge (legendary pack path) |
| `story` | CCL-aware story brief |

Wild Magic is intentional theater — not a bug and not a living KPI failure. Tier labels like `MINOR SETBACK` come from the dice event table.

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
