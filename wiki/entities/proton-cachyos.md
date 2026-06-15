---
type: entity
title: Proton-CachyOS
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Proton-CachyOS

Type: SYSTEM

## From [[drive-research-ultimate-linux-workstation-tuning-blueprint|drive-research-ultimate-linux-workstation-tuning-blueprint]] (2026-06-08)
- Historically relied on esync or fsync.
- Proton 11 natively supports NTsync.
- Steam Launch Options configure its execution.
- includes NTSync
- Natively supports NTsync.
- Execution is confined strictly to Cores 0-15 (CCD0) via cgroups v2.
- Requires specific Steam Launch Options.
