---
type: entity
title: /proc/<pid>/statm
created: 2026-06-08
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# /proc/<pid>/statm

Type: SYSTEM

## From [[drive-research-architecting-a-linux-task-manager-design-principl|drive-research-architecting-a-linux-task-manager-design-principl]] (2026-06-08)
- Contains raw numerical memory utilization data (virtual, resident, shared, etc.) measured specifically in memory pages.
- Is preferred over the status file for high-performance task managers due to faster parsing.
- Contains critical columns for memory calculation: size, resident, and shared.

## From [[gzmo-soul-merged-new-part2-micro05|gzmo-soul-merged-new-part2-micro05]] (2026-06-09)
- Provides memory consumption in Pages.
