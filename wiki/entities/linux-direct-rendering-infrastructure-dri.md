---
type: entity
title: Linux Direct Rendering Infrastructure (DRI)
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Linux Direct Rendering Infrastructure (DRI)

Type: SYSTEM

## From [architectural-strategy-for-stealthy-portable-cli-a](/entities/architectural-strategy-for-stealthy-portable-cli-a.md) (2026-06-08)
- Heavily interacted with by the ROCm ecosystem on Linux.
- Involved in the verification method for AMD ROCm.
- One of the architectures for which cross-platform functionality is required.
- Uses /tmp/_MEIxxxxxx for temporary directory extraction by PyInstaller.
- Utilizes auditd for logging subprocess execution.
