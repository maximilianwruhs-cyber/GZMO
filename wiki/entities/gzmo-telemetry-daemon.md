---
type: entity
title: GZMO telemetry daemon
created: 2026-06-10
updated: 2026-06-10
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# GZMO telemetry daemon

Type: SYSTEM

## From [[phantom-drive-autonomous-llm-deployment-architect-micro02|phantom-drive-autonomous-llm-deployment-architect-micro02]] (2026-06-10)
- A background service/daemon used for telemetry.
- Must be cryptographically bound to the boot.sh script to prevent becoming an orphaned process.
