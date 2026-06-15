---
type: entity
title: mdev
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---


# mdev

Type: TOOL

## From [[drive-research-automating-linux-hardware-detection-micro01|drive-research-automating-linux-hardware-detection-micro01]] (2026-06-09)
- Highly simplified component of the BusyBox suite, frequently utilized in deeply embedded IoT systems or independent distributions.
- Requires the kernel to be compiled with the CONFIG_UEVENT_HELPER option enabled.
- Relies on the kernel spawning a fresh instance of the application for every single hardware event, passing event information via standard input arguments.
