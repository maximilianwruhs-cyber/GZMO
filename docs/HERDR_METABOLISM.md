# herdr ↔ GZMO Metabolism

**Status:** Unpark Wave 1.1 (2026-07-19)  
**Plugin:** `integrations/herdr-gzmo-metabolism/`  
**Link script:** `scripts/herdr-metabolism-link.sh`  
**Check:** `scripts/herdr-metabolism-check.sh` → `data-next/herdr-metabolism/`  
**Related:** [DISTILL_COLD_CHAIN.md](./DISTILL_COLD_CHAIN.md), [SYNAPSE_EVENT_OWNERSHIP.md](./SYNAPSE_EVENT_OWNERSHIP.md), [UNPARK_ROADMAP.md](./UNPARK_ROADMAP.md)

## What it does

herdr is an optional operator shell. The plugin attaches **product/living memory MCP** and forces a **takeaway → `gzmo session close` → distill enqueue** ritual when you intentionally close a session.

It does **not** auto-distill on every pane close.

## Install / link

```bash
# herdr must be on PATH
bash scripts/herdr-metabolism-link.sh
```

Actions:

| Action | Purpose |
|--------|---------|
| `ensure-mcp` | Wire `gzmo-memory` MCP into the workspace |
| `session-close` | Takeaway → `gzmo session close --takeaway …` |
| `status` | Plugin / GZMO metabolism status |
| pane `close-ritual` | Interactive overlay for the takeaway |

Optional env drop: `$(herdr plugin config-dir gzmo.metabolism)/env`  
Takeaway file: `…/takeaway.txt` (one durable line)

## Close ritual (canonical)

**Piggyback doctrine (takeaway side-effect):** run this at the end of work you were doing anyway. It is not a reason to open Cursor for memory practice. Reminders live in `.github/pull_request_template.md` and optional `scripts/takeaway-side-effect-remind.sh --install-hook` — they never auto-distill.

```bash
# Interactive
herdr plugin pane open --plugin gzmo.metabolism --entrypoint close-ritual

# Non-interactive (lab / local next)
TAKEAWAY='durable fact for the vault' \
  herdr plugin action invoke gzmo.metabolism.session-close

# Living host enqueue (CT101) — same ritual, SSH session close, no --now
bash scripts/herdr-living-enqueue.sh
# or:
TAKEAWAY='durable fact' \
  bash integrations/herdr-gzmo-metabolism/scripts/session-close.sh --living --takeaway 'durable fact'

# CLI without herdr (same ritual)
gzmo session close --takeaway 'durable fact for the vault'
```

Under the hood: `gzmo session close [--session ID] --takeaway '…' [--now]`. Prefer living-host enqueue; avoid `--now` on the workstation while CT101 owns overnight. `--living` refuses `--now` and dual-writer (`gzmo-serve` active).

## `pane.closed` is soft-miss only

`on-pane-closed.sh` appends a row to the plugin state `missed-close.jsonl` and exits 0. It **never** enqueues distill. If sessions never metabolize, operators skipped the ritual — check that log, do not “fix” by auto-distilling every close.

## Relation to Synapse / Pi

| Path | Distill trigger |
|------|-----------------|
| herdr session-close | `gzmo session close` → distill queue |
| Pi `session_end` | synapse-notifier + daemon poll — [SYNAPSE_EVENT_OWNERSHIP.md](./SYNAPSE_EVENT_OWNERSHIP.md) |
| CLI takeaway | same close ritual without herdr |

Do not invent a fourth close path. Product MCP attach vs living CT101 attach: [PI_LIVING_STACK.md](./PI_LIVING_STACK.md).

## Unpark policy

Wave 1.1 — demable via:

```bash
bash scripts/herdr-metabolism-demo.sh   # link + takeaway → enqueue (lab, no --now)
bash scripts/herdr-metabolism-check.sh  # includes close-ritual evidence row
```

Soft living-readiness row when herdr is absent (HOLD), PASS when link + close-ritual ok.
