---
name: character
description: Choose/list OpenClaw personas from openclaw-agents without wiping GZMO AGENTS.md. Telegram /character or /skill character.
user-invocable: true
command-dispatch: tool
command-tool: character
command-arg-mode: raw
metadata:
  openclaw:
    requires:
      bins: ["bash"]
---

# character

Slash `/character` is dispatched **directly** to the `character` tool (no model).

If you are invoked without slash dispatch, call the **character** tool once:

- `command`: user args (`who` | `list` | `search …` | `<slug>`). Empty → `who`.

Do **not** `exec read …/SKILL.md`. Do **not** invent other tool ids.
Never run upstream `openclaw-agents/install.sh`.

After a successful persona install, tell Max to send `/new`.
