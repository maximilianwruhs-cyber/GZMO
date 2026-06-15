---
type: entity
title: Hybrid MBR
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Hybrid MBR

Type: CONCEPT

## From [[architecting-the-minimalist-linux-desktop-a-compa-part2|architecting-the-minimalist-linux-desktop-a-compa-part2]] (2026-06-08)
- It is a severe and controversial deviation from the official Intel EFI GPT standard.
- It manually modifies Sector 0 to include up to three actual MBR partition entries alongside a shrunken 0xEE partition.
- Hybrid MBRs are inherently fragile and widely considered dangerous in modern computing.
