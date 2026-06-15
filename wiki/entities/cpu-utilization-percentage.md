---
type: entity
title: CPU Utilization Percentage
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# CPU Utilization Percentage

Type: CONCEPT

## From [[drive-research-architecting-a-linux-task-manager-design-principl|drive-research-architecting-a-linux-task-manager-design-principl]] (2026-06-08)
- Calculated by the task manager using the mathematical delta between two distinct points in time.
- Formula: 100 × ((total_time / Hertz) / seconds_alive).
- For instantaneous usage, involves dividing the delta of total_time by the delta of system_uptime across the last UI refresh interval.
