---
type: entity
title: AES-XTS
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# AES-XTS

Type: CONCEPT

## From [architecting-the-minimalist-linux-desktop-a-compa-part1](/entities/architecting-the-minimalist-linux-desktop-a-compa-part1.md) (2026-06-08)
- Modern LUKS deployments default to the AES cipher utilizing XTS mode (aes-xts-plain64).
- Exceptionally resilient from a data integrity and power-loss standpoint.
- Each 16-byte sub-block is encrypted independently based on its specific sector position and the master key.
