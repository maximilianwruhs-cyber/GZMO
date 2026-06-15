---
type: entity
title: tmpfs
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# tmpfs

Type: SYSTEM

## From [[architecting-the-minimalist-linux-desktop-a-compa-part1|architecting-the-minimalist-linux-desktop-a-compa-part1]] (2026-06-08)
- A purely RAM-backed tmpfs overlay has a WA of 1.03.
- Used by Alpine Linux for its overlay setup.
- High-churn directories like /var and /tmp can be migrated into RAM-backed tmpfs mounts to achieve SSD-like responsiveness on USB media.
