---
type: entity
title: GRUB2
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# GRUB2

Type: SYSTEM

## From [[architecting-the-minimalist-linux-desktop-a-compa-part2|architecting-the-minimalist-linux-desktop-a-compa-part2]] (2026-06-08)
- It is the core image of the GRUB2 bootloader, which is an absolute necessity for Legacy BIOS booting.
- Ventoy leaves a deliberate 1MB gap at the beginning of the disk to hold its core image.
- Engineers can manually construct a hybrid boot USB using GRUB2.
