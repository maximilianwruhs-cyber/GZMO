---
type: entity
title: Checkpoint Manager
created: 2026-06-08
updated: 2026-06-08
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# Checkpoint Manager

Type: SYSTEM

## From [[drive-research-hermes-compression-and-bol-architecture|drive-research-hermes-compression-and-bol-architecture]] (2026-06-08)
- Governs the physical safety of the host machine in Hermes.
- Implements an autonomous shadow version-control layer.
- Intercepts agent tool invocations designed to modify, overwrite, or delete files.
- Interfaces with a hidden Git repository located at ~/.hermes/checkpoints/store/.

## From [[drive-research-hermes-system-untersuchung-und-erweiterung|drive-research-hermes-system-untersuchung-und-erweiterung]] (2026-06-08)
- Acts as an automatic safety net anticipating destructive operations.
- Uses a hidden version control architecture.
- Maintains a separate shadow Git repository in ~/.hermes/checkpoints/store/.
