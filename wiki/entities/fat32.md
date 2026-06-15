---
type: entity
title: FAT32
created: 2026-06-08
updated: 2026-06-08
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# FAT32

Type: CONCEPT

## From [[architecting-the-minimalist-linux-desktop-a-compa-part2|architecting-the-minimalist-linux-desktop-a-compa-part2]] (2026-06-08)
- It is the baseline file system that all UEFI implementations must support.
- It has a hard-coded, mathematically insurmountable limitation: it cannot store any individual file larger than 4 Gigabytes minus 1 byte.
- UEFI firmware will only recognize FAT32 partitions when hunting for .efi bootloaders.

## From [[architectural-strategy-for-stealthy-portable-cli-a|architectural-strategy-for-stealthy-portable-cli-a]] (2026-06-08)
- Any file written to a FAT32 USB drive is globally readable, writable, and executable by any user, process, or malware currently operating on the host machine.
- These filesystems lack POSIX/NTFS file permission security (no ACLs).
