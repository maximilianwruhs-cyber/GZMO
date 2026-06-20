---
type: entity
title: Google Cloud Filestore (Zonal tier)
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Google Cloud Filestore (Zonal tier)

Type: TOOL

## From [openclaw-part2](/entities/openclaw-part2.md) (2026-06-08)
- The OpenClaw Gateway relies on Filestore for state persistence.
- The agent is permitted to perform write operations exclusively within the designated, isolated Filestore workspace mounts.
- Bypasses standard cloud storage limitations by using 'Custom Performance' tier or Hyperdisk Balanced for massive IOPS.
- Provides a robust, managed Network File System (NFS) for GKE.
- Allows multiple pods ReadWriteMany access for shared agent workspaces.
- The Zonal service tier is heavily recommended for enterprise OpenClaw deployments.
- A GCP solution for Storage & Memory Optimization for GKE.
- Uses the Custom Performance feature to provision massive IOPS for localized SQLite searches.
- Keeps the storage capacity footprint minimal.
