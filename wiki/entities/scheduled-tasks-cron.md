---
type: entity
title: Scheduled Tasks (Cron)
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Scheduled Tasks (Cron)

Type: TOOL

## From [[openclaw-deep-research-part11-micro06|openclaw-deep-research-part11-micro06]] (2026-06-09)
- The Gateway's built-in scheduler.
- Used for precise timing.
- Supports expressions and one-shot reminders.
- Gateway's built-in scheduler for precise timing.
- Persists jobs, wakes the agent at the right time, and can deliver output to a chat channel or webhook endpoint.
- Supports one-shot reminders, recurring expressions, and inbound webhook triggers.
- Used when precise timing or isolated execution is needed.
- All cron executions create task records.
