---
type: entity
title: F2FS
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# F2FS

Type: CONCEPT

## From [architecting-the-minimalist-linux-desktop-a-compa-part1](/entities/architecting-the-minimalist-linux-desktop-a-compa-part1.md) (2026-06-08)
- A standard journaling or log-structured filesystem that can be used for the Upperdir in OverlayFS.
- Flash-Friendly File System, engineered by Samsung in 2012.
- Explicitly designed to interface with the internal geometry of NAND flash memory.
- Utilizes a log-structured file system (LFS) approach, dividing the volume into fixed 2 MB segments.
