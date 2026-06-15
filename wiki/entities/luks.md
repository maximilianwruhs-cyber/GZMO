---
type: entity
title: LUKS
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# LUKS

Type: SYSTEM

## From [[architecting-the-minimalist-linux-desktop-a-compa-part1|architecting-the-minimalist-linux-desktop-a-compa-part1]] (2026-06-08)
- Linux Unified Key Setup, provides block-level encryption.
- Operates completely transparently beneath the filesystem layer.
- Modern LUKS deployments default to the AES cipher utilizing XTS mode (aes-xts-plain64).
