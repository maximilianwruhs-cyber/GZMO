---
type: entity
title: sysfsutils
created: 2026-06-09
updated: 2026-06-09
sources: 4
tags: []
status: draft
gzmo_synthetic: true
---




# sysfsutils

Type: TOOL

## From [[the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro04|the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro04]] (2026-06-09)
- Used to set granular permissions for hardware-level monitoring files.
- Allows agent processes to read microjoule consumption.
- Assigns ownership of RAPL sysfs paths to a dedicated power group.

## From [[the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro05|the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro05]] (2026-06-09)
- Used to check RAPL group and sysfs permissions.

## From [[the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro08|the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro08]] (2026-06-09)
- Used for configuring Intel RAPL.
- Enables permanent provision of energy data without root privileges.

## From [[the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro09|the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro09]] (2026-06-09)
- Installed to persist permission changes across reboots.
- Used to configure granular non-root access to /sys/class/powercap.
