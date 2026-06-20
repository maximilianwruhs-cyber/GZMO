---
type: entity
title: Cluster Launch Control (CLC) Dynamic Scheduling
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Cluster Launch Control (CLC) Dynamic Scheduling

Type: CONCEPT

## From [drive-research-imagine-creating-sm120-according-to-our-progress](/entities/drive-research-imagine-creating-sm120-according-to-our-progress.md) (2026-06-08)
- The Blackwell architecture introduces Cluster Launch Control (CLC) to drive a hardware-supported work-stealing loop.
- An elected thread within the cluster's leader warp executes an asynchronous try-cancel instruction to request a block cancellation from the hardware scheduler.
- The hardware writes a 16-byte response packet into shared memory, decrementing the transaction barrier.
- Once resolved, the threads execute clusterlaunchcontrol.query_cancel to decode the packet, fetch the logical coordinates of the stolen work tile, and branch directly to the mainloop execution—completely mitigating idle cycles.
- Cluster Launch Control (CLC) Dynamic Scheduling.
- Static scheduling algorithms leave workstation SM resources highly vulnerable to load imbalances and display task interruptions, inducing 'CTA drift' where early-finishing thread blocks sit idle.
