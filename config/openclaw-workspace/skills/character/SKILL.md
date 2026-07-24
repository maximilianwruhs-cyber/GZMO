---
name: character
description: Choose/list OpenClaw personas from openclaw-agents without wiping GZMO AGENTS.md. Telegram /character.
user-invocable: true
metadata:
  openclaw:
    requires:
      bins: ["bash"]
---

# Character chooser (GZMO-safe)

You are handling a **Telegram `/character`** (or `/skill character`) request.

## What to run

Always use **exec** with this script (never upstream `openclaw-agents/install.sh`):

```bash
export OPENCLAW_CHARACTER_FORCE=1
CHOOSER="$HOME/github-clone/GZMO/scripts/openclaw-choose-character.sh"
```

| User args | Command |
|-----------|---------|
| empty / `help` | Print short help + `bash "$CHOOSER" who` |
| `who` / `status` | `bash "$CHOOSER" who` |
| `list` | `bash "$CHOOSER" list` (truncate to ~40 lines if huge) |
| `search <q>` | `bash "$CHOOSER" search <q>` |
| `<slug>` (e.g. `glados`) | `bash "$CHOOSER" <slug>` |

## Reply rules

1. Run the command; paste the useful output (persona name/emoji, OK/REFUSE).
2. After a successful install, tell Max to send **`/new`** so the new SOUL/IDENTITY load.
3. Never claim the GZMO ecosystem was removed — the chooser re-syncs markers.
4. Never Qdrant upsert / start `gzmo-serve` / overwrite AGENTS by hand.

## Examples

- `/character who`
- `/character search duck`
- `/character rubber-duck`
- `/character glados`
