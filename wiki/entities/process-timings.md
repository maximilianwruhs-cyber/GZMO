---
type: entity
title: Process Timings
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Process Timings

Type: CONCEPT

## From [[drive-research-architecting-a-linux-task-manager-design-principl|drive-research-architecting-a-linux-task-manager-design-principl]] (2026-06-08)
- Extracted from the /proc/<pid>/stat file.
- Includes utime (user mode), stime (kernel mode), cutime (child user mode), cstime (child kernel mode), and starttime.
