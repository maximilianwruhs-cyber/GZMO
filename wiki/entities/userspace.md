---
type: entity
title: userspace
created: 2026-06-09
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---



# userspace

Type: CONCEPT

## From [drive-research-automating-linux-hardware-detection-micro01](/entities/drive-research-automating-linux-hardware-detection-micro01.md) (2026-06-09)
- Daemons execute the policies to name, configure, and expose underlying devices to application software.
- Receives uevents from the kernel.
- Consumes hardware discovery information.

## From [drive-research-automating-linux-hardware-detection-micro03](/entities/drive-research-automating-linux-hardware-detection-micro03.md) (2026-06-09)
- Drivers or utilities attempt to bypass kernel abstractions.
- Developing hardware drivers directly in userspace is frequently attempted.
- Exposes the I2C bus to the application.
- A flawed userspace script can manipulate other devices.
- A userspace master device can lock a peripheral.
