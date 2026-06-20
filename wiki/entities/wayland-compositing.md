---
type: entity
title: Wayland compositing
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Wayland compositing

Type: CONCEPT

## From [drive-research-ultimate-linux-workstation-tuning-blueprint](/entities/drive-research-ultimate-linux-workstation-tuning-blueprint.md) (2026-06-08)
- zero-latency
- concurrent with LLM inferencing
- Display server protocol used by Bazzite.
- Historical reliance on implicit synchronization caused tearing.
- Requires explicit synchronization via linux-drm-syncobj-v1.
- supports Explicit Sync
- used with KDE Plasma
