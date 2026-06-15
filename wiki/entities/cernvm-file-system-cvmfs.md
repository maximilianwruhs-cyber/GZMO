---
type: entity
title: CernVM File System (CVMFS)
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# CernVM File System (CVMFS)

Type: SYSTEM

## From [[architecting-the-minimalist-linux-desktop-a-compa-part1|architecting-the-minimalist-linux-desktop-a-compa-part1]] (2026-06-08)
- Distributes software to the global LHC computing infrastructure.
- Successfully relies on an OverlayFS union mount to manage over 5 billion files across 100,000 worker nodes.
- Utilizes content-addressable storage and Merkle trees to optimize read-only performance.
