---
type: entity
title: Ext4
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Ext4

Type: CONCEPT

## From [[architecting-the-minimalist-linux-desktop-a-compa-part1|architecting-the-minimalist-linux-desktop-a-compa-part1]] (2026-06-08)
- A standard journaling or log-structured filesystem that can be used for the Upperdir in OverlayFS.
- Default filesystem in most Linux distributions.
- Utilizes a journaling system to prevent structural corruption.
- Exhibits a write amplification factor of approximately 2x to 30x on standard hardware in live environments.
