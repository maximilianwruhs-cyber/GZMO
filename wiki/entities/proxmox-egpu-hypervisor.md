---
type: entity
title: Proxmox eGPU hypervisor
created: 2026-06-08
updated: 2026-06-08
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# Proxmox eGPU hypervisor

Type: SYSTEM

## From [[architectural-blueprints-for-sovereign-frankenmoe-part2|architectural-blueprints-for-sovereign-frankenmoe-part2]] (2026-06-08)
- The final architecture is optimized for a specific Proxmox eGPU hypervisor setup.
- Proxmox Virtualization & Hardware Topology is a section in the document.
- The entire 38 GB model is unified inside graphics memory by passing all three physical GPUs through to a single high-performance guest environment.

## From [[drive-research-so-what-is-your-final-model-constellation|drive-research-so-what-is-your-final-model-constellation]] (2026-06-08)
- Optimized to prevent mathematical quantization decay and CPU-bound memory bus bottlenecks.
- Used for the final architecture setup.
