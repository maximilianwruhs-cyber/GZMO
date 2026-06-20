---
type: entity
title: EFI System Partition (ESP)
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# EFI System Partition (ESP)

Type: CONCEPT

## From [architecting-the-minimalist-linux-desktop-a-compa-part2](/entities/architecting-the-minimalist-linux-desktop-a-compa-part2.md) (2026-06-08)
- UEFI firmware reads the partition table, locates a dedicated ESP, and searches for pre-compiled bootloader applications.
- It is mandated to possess the specific partition type GUID C12A7328-F81F-11D2-BA4B-00A0C93EC93B.
- It is typically formatted as FAT32.
