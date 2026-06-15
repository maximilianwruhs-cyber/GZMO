---
type: entity
title: Ultra Path Interconnect (UPI)
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Ultra Path Interconnect (UPI)

Type: CONCEPT

## From [[drive-research-llamacpp-optimization-blueprint-micro02|drive-research-llamacpp-optimization-blueprint-micro02]] (2026-06-09)
- Data must traverse the QuickPath Interconnect (QPI) or UPI if a thread on CPU 0 attempts to access memory on CPU 1's RAM bank.
- Cross-NUMA latency destroys token generation speed.
