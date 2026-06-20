---
type: entity
title: Raspberry Pi 3
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Raspberry Pi 3

Type: SYSTEM

## From [architecting-the-minimalist-linux-desktop-a-compa-part2](/entities/architecting-the-minimalist-linux-desktop-a-compa-part2.md) (2026-06-08)
- Its on-board bootloader strictly requires an MBR partition table to load its firmware payload.
- Cannot parse GPT data structures during initial hardware bootstrap.
- Requires falling back to pure MBR layouts or customized Hybrid MBRs for modern OS deployment.
