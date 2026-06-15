---
type: entity
title: NAND flash memory
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# NAND flash memory

Type: CONCEPT

## From [[architecting-the-minimalist-linux-desktop-a-compa-part1|architecting-the-minimalist-linux-desktop-a-compa-part1]] (2026-06-08)
- F2FS was explicitly engineered to interface with the internal geometry of NAND flash memory.
- Operates on an "erase-before-write" principle.
- Modifying a 4 KiB file requires reading the entire 1 MiB block into memory, erasing the physical block, updating the 4 KiB segment, and writing the entire 1 MiB block back.
