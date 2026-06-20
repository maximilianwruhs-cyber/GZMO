---
type: entity
title: Secure Boot
created: 2026-06-08
updated: 2026-06-08
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# Secure Boot

Type: CONCEPT

## From [architecting-the-minimalist-linux-desktop-a-compa-part1](/entities/architecting-the-minimalist-linux-desktop-a-compa-part1.md) (2026-06-08)
- A system feature that can restrict nvidia.ko.
- Can lead to a failure state in the primary path.

## From [architecting-the-minimalist-linux-desktop-a-compa-part2](/entities/architecting-the-minimalist-linux-desktop-a-compa-part2.md) (2026-06-08)
- It is an advanced cryptographic security measure implemented by UEFI.
- It ensures that only bootloaders signed with trusted digital certificates can execute.
- Utilizing CSM inherently disables this feature.
- Windows 11 mandates strict adherence to it.
- A protocol related to UEFI.
- Its mandate contributes to the diminishing necessity of MBR-based legacy boot.
