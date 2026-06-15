---
type: entity
title: Deferred Deletion (Command Buffer)
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Deferred Deletion (Command Buffer)

Type: CONCEPT

## From [[drive-research-rust-ecs-cache-optimization-research|drive-research-rust-ecs-cache-optimization-research]] (2026-06-08)
- A preferred synchronization pattern for real-time runtimes.
- Agents write deletion requests to a lock-free multi-producer, single-consumer (MPSC) queue.
- A single-threaded system sweep processes the queue sequentially.
