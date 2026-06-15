---
type: entity
title: Windows SmartScreen
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Windows SmartScreen

Type: SYSTEM

## From [[architectural-strategy-for-stealthy-portable-cli-a|architectural-strategy-for-stealthy-portable-cli-a]] (2026-06-08)
- Even the safest behavior will trigger Windows SmartScreen if the binary is unsigned.
- One of the architectures for which cross-platform functionality is required.
- Uses %TEMP%_MEIxxxxxx for temporary directory extraction by PyInstaller.
- Utilizes ntdll.dll for user-land API monitoring hooks.
