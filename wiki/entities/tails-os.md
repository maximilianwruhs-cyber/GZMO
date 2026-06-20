---
type: entity
title: Tails OS
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Tails OS

Type: SYSTEM

## From [architecting-the-minimalist-linux-desktop-a-compa-part1](/entities/architecting-the-minimalist-linux-desktop-a-compa-part1.md) (2026-06-08)
- The Amnesic Incognito Live System, a privacy-focused memory vault.
- Rejects both the Fully Writable Rootfs and the Global OverlayFS models.
- Utilizes a read-only SquashFS core paired with a purely RAM-based overlay (tmpfs) to handle all general system state.
