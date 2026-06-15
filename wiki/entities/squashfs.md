---
type: entity
title: SquashFS
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# SquashFS

Type: TOOL

## From [[architecting-the-minimalist-linux-desktop-a-compa-part1|architecting-the-minimalist-linux-desktop-a-compa-part1]] (2026-06-08)
- A mechanism used to heavily compress a read-only rootfs in Live OS.
- Typically used for the Lowerdir in OverlayFS.
- An immutable, read-only SquashFS image is used for the base OS in OverlayFS.
- Tails OS uses a read-only SquashFS core.
- Mathematically impossible for a sudden power loss to corrupt the fundamental operating system binaries when backed by SquashFS.
