---
type: entity
title: PROTON_ENABLE_WAYLAND
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# PROTON_ENABLE_WAYLAND

Type: SYSTEM

## From [[drive-research-linux-gaming-and-ai-build-guide-micro01|drive-research-linux-gaming-and-ai-build-guide-micro01]] (2026-06-09)
- Has entirely replaced X11 as the modern default on GNOME, KDE Plasma, and major distributions.
- Wayland's architecture collapses the display pipeline into a single unified compositor.
- NVIDIA's proprietary Linux drivers historically clashed violently with Wayland compositors.
- This environment variable may require manual overrides for Wayland on NVIDIA.
- Used to enable Wayland support within the Proton compatibility layer.
