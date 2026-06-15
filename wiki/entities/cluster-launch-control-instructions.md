---
type: entity
title: Cluster Launch Control instructions
created: 2026-06-10
updated: 2026-06-10
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Cluster Launch Control instructions

Type: TOOL

## From [[optimizing-nvidia-blackwell-sm120-part2-micro03|optimizing-nvidia-blackwell-sm120-part2-micro03]] (2026-06-10)
- Includes clusterlaunch.span_179 and clusterlaunchcontrol.span_181
- Includes try_cancel.async instruction
- Hardware-supported implementation of dynamic persistent tile scheduling
- Uses clusterlaunch instructions
- Enables work-stealing via try-cancel instructions
