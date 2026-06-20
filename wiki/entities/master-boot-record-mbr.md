---
type: entity
title: Master Boot Record (MBR)
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Master Boot Record (MBR)

Type: CONCEPT

## From [architecting-the-minimalist-linux-desktop-a-compa-part2](/entities/architecting-the-minimalist-linux-desktop-a-compa-part2.md) (2026-06-08)
- The entire partition table is confined to a diminutive 64-byte segment within the 512-byte Sector 0.
- An MBR-formatted drive is strictly mathematically limited to a maximum of four primary partitions.
- The MBR scheme is mathematically incapable of natively addressing storage space beyond the 2 TB threshold.
- Ancient motherboards may demand strict adherence to CHS geometry encodings within it.
- The on-board bootloader of the Raspberry Pi 3 strictly requires an MBR partition table.
- Microsoft Windows actively rejects UEFI boot with an MBR-partitioned disk during installation.
