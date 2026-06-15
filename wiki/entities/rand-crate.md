---
type: entity
title: rand crate
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# rand crate

Type: TOOL

## From [[deterministic-polyhedral-dynamics-in-rust-game-eng|deterministic-polyhedral-dynamics-in-rust-game-eng]] (2026-06-08)
- Used in a standard Rust environment for PRNG.
- A thread-local generator evaluates a bounded range.
- Example: rand::thread_rng().gen_range(1..=6) for a d6.
