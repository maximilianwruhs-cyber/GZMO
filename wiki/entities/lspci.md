---
type: entity
title: lspci
created: 2026-06-09
updated: 2026-06-10
sources: 3
tags: []
status: draft
gzmo_synthetic: true
---




# lspci

Type: TOOL

## From [drive-research-automating-linux-hardware-detection-micro03](/entities/drive-research-automating-linux-hardware-detection-micro03.md) (2026-06-09)
- Output is harvested by hw-probe.

## From [phantom-drive-autonomous-llm-deployment-architect-micro01](/entities/phantom-drive-autonomous-llm-deployment-architect-micro01.md) (2026-06-09)
- Used for driver-agnostic hardware interrogation.
- Scans PCI buses for NVIDIA vendor ID (10de).
- Confirms physical presence of GPU without invoking proprietary drivers.

## From [drive-research-automating-linux-hardware-detection-micro02](/entities/drive-research-automating-linux-hardware-detection-micro02.md) (2026-06-10)
- Interfaces with the PCI bus to expose connected devices
- Supports -k and -v flags
