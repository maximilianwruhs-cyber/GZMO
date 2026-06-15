---
type: entity
title: Khronos Vulkan Installable Client Driver (ICD) loader
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Khronos Vulkan Installable Client Driver (ICD) loader

Type: SYSTEM

## From [[architecting-the-minimalist-linux-desktop-a-compa-part1|architecting-the-minimalist-linux-desktop-a-compa-part1]] (2026-06-08)
- A highly performant backend.
- Used by the llama.cpp inference engine when NVK user-space driver is injected.
- Designed to be configurable via environment variables.
- Can be manipulated via environment variables to hijack the driver loading process.
- Uses VK_DRIVER_FILES as the primary variable to force the Vulkan loader to use specific driver JSON manifest files.
