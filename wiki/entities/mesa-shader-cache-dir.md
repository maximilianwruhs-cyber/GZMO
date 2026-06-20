---
type: entity
title: MESA_SHADER_CACHE_DIR
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# MESA_SHADER_CACHE_DIR

Type: CONCEPT

## From [architecting-the-minimalist-linux-desktop-a-compa-part1](/entities/architecting-the-minimalist-linux-desktop-a-compa-part1.md) (2026-06-08)
- Specifies where the on-disk cache should be stored for read-write operations.
- In a Flatpak or AppImage, it must be redirected to a writable location within the application's sandbox.
- Prevents permissions errors on the arbitrary host.
