---
type: entity
title: /proc/<pid>/stat
created: 2026-06-08
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# /proc/<pid>/stat

Type: SYSTEM

## From [drive-research-architecting-a-linux-task-manager-design-principl](/entities/drive-research-architecting-a-linux-task-manager-design-principl.md) (2026-06-08)
- A densely packed, single-line file with raw status information.
- Includes CPU consumption ticks, run state, and parent PID (PPID).
- Contains critical fields for CPU utilization calculation: utime, stime, cutime, cstime, and starttime.
- Is parsed by task managers.
- Contains system-wide CPU utilization metrics (aggregated and per-core) and cumulative "jiffies" (clock ticks) since boot.
- Is parsed to establish the global context of the machine.

## From [gzmo-soul-merged-new-part2-micro05](/entities/gzmo-soul-merged-new-part2-micro05.md) (2026-06-09)
- Provides system-wide CPU metrics (Jiffies).
- Logbook of the CPU, showing accumulated 'Jiffies'.
- Provides process-specific ticks (utime, stime).
- A goldmine for admins, containing 52 fields.
- Fields 14 (utime), 15 (stime), and 22 (starttime) are important.
