# Pantheon felt honesty — 2026-07-22

Finish the theater front door so `--json` and demos are readable.

## Problem

`gzmo chaos skill dice d20 --json` printed **cascade child** evidence (`skill: "define"`) and dropped the D20 roll framing. Stale `target/release/gzmo` ignored `chaos skill` and opened chat.

## Fix

- Dice evidence envelope: `skill: "dice"`, `roll`/`max`, full `display_plain`, `cascade_skill` + `cascade`
- `chaos_skill_cmd`: stamp `invoked`; wrap mismatched nested evidence
- `scripts/pi/chaos_skill.sh`: probe binaries; refuse chat-fallthrough builds
- Docs: Wild Magic explained in [PANTHEON_DEMO.md](../docs/PANTHEON_DEMO.md)

## Verify

```bash
bash scripts/pantheon-ritual-demo.sh          # GREEN + felt-latest.md
bash scripts/pi/chaos_skill.sh dice d20 --json  # skill=dice, roll=N, cascade_skill=…
```
