---
type: entity
title: Inter-Integrated Circuit (I2C) protocol
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---


# Inter-Integrated Circuit (I2C) protocol

Type: CONCEPT

## From [[drive-research-automating-linux-hardware-detection-micro03|drive-research-automating-linux-hardware-detection-micro03]] (2026-06-09)
- A ubiquitous, asynchronous two-wire serial communication standard.
- Heavily utilized for interfacing with onboard sensors, EEPROMs, and microcontrollers.
- Userspace drivers can expose the entire I2C bus.
- Lacks robust, hardware-level isolation mechanisms.
- A malformed query can cause address conflicts or poor signal integrity.
- A flawed userspace script can manipulate other critical devices.
- A failure to transmit proper STOP conditions can lock a peripheral slave device.
