---
type: entity
title: x86_64-unknown-linux-musl
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# x86_64-unknown-linux-musl

Type: CONCEPT

## From [[architecting-the-minimalist-linux-desktop-a-compa-part1|architecting-the-minimalist-linux-desktop-a-compa-part1]] (2026-06-08)
- The specific architecture Rust developers must target for Alpine static binaries.
- It is a C standard library.
- It was introduced in 2011 as a lightweight alternative to glibc.
- It has a strict focus on minimalism, correctness, and unwavering adherence to POSIX and ISO C standards.
- Attempting to execute glibc-precompiled native bindings on it results in missing symbols or segmentation faults.
- Developers install libc6-compat to force Node modules to execute correctly on it.
- The core libc of Alpine Linux.
