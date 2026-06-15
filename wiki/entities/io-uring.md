---
type: entity
title: io_uring
created: 2026-06-08
updated: 2026-06-10
sources: 5
tags: []
status: draft
gzmo_synthetic: true
---





# io_uring

Type: CONCEPT

## From [[refactoring-gzmo-daemon-for-native-bun-high-perfor|refactoring-gzmo-daemon-for-native-bun-high-perfor]] (2026-06-08)
- Leveraged by Bun.write and Bun.file on Linux.
- Enables true zero-latency reads and writes.
- Bypasses event loop friction.

## From [[drive-research-ultimate-linux-workstation-tuning-blueprint|drive-research-ultimate-linux-workstation-tuning-blueprint]] (2026-06-08)
- Modern Linux interface for I/O.
- Utilizes shared memory ring buffers.
- Virtually eliminates system call overhead.
- maturing well on Linux
- provides faster & more flexible I/O

## From [[drive-research-advanced-typescript-execution-in-the-bun-runtime-micro03|drive-research-advanced-typescript-execution-in-the-bun-runtime-micro03]] (2026-06-09)
- On Linux operating systems, Bun bypasses traditional blocking I/O by heavily leveraging io_uring—a highly advanced, asynchronous Linux kernel API that allows applications to queue I/O operations without incurring the massive overhead of system call context switching.
- Eliminating runtime I/O latency via compile-time execution macros.

## From [[drive-research-bun-typescript-performance-tips-micro03|drive-research-bun-typescript-performance-tips-micro03]] (2026-06-09)
- A highly advanced, asynchronous Linux kernel API.
- Bun heavily leverages io_uring on Linux.
- Allows applications to queue I/O operations without massive overhead.

## From [[high-performance-typescript-execution-and-architec-part1-micro03|high-performance-typescript-execution-and-architec-part1-micro03]] (2026-06-10)
- An advanced, asynchronous Linux kernel API.
- Allows applications to queue I/O operations without massive system call context switching overhead.
