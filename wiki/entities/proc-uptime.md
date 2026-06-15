---
type: entity
title: /proc/uptime
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# /proc/uptime

Type: SYSTEM

## From [[drive-research-architecting-a-linux-task-manager-design-principl|drive-research-architecting-a-linux-task-manager-design-principl]] (2026-06-08)
- Provides the total system uptime in seconds and the total time spent executing the idle process.
- Is parsed to establish the global context of the machine.
- Is used to provide a temporal anchor for CPU utilization calculations.
