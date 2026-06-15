---
type: entity
title: OverlayFS Penalty
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# OverlayFS Penalty

Type: CONCEPT

## From [[architecting-the-minimalist-linux-desktop-a-compa-part1|architecting-the-minimalist-linux-desktop-a-compa-part1]] (2026-06-08)
- Refers to the massive macro-level write amplification introduced by the copy_up mechanism.
- The first write to any unmodified system file is astronomically expensive due to this.
- A journaling configuration combined with an OverlayFS structure can consume exponentially more physical NAND writes than optimized configurations.
- Dramatically alters the power-loss failure domain, making it highly attractive for embedded appliances and secure memory vaults.
- The kernel provides the fsync mount option to tune the balance between performance and durability.
