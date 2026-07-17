---
name: handoff
description: Compress the session into a focused handoff markdown for the next agent or session — purpose, context, suggested skills, artifact pointers.
triggers:
  - handoff
requires_evidence: false
---

# Handoff

Produce a compact handoff document so the next session can continue without replaying this whole conversation.

## Required sections

1. **Purpose of next session** — one paragraph
2. **Done so far** — bullets
3. **Not done / blockers** — bullets
4. **Key decisions** — bullets (only non-obvious)
5. **Suggested workflow skills** — e.g. `tdd`, `diagnose`, `grill`
6. **Artifacts** — paths to files, tests, configs touched
7. **First next action** — single concrete step

## How to persist

- Write the markdown with `file_write` under the configured handoffs directory (default `data-next/handoffs/`), filename `handoff-<session>-<utc>.md`.
- Also call `memory_record` with a one-line summary pointing at that path when vault tools are available.
- Keep the body under ~80 lines. No raw transcript dumps.

## Rules

- Prefer pointers over pasted code.
- Do not start new feature work inside handoff — document and stop.
