---
type: entity
title: Bun.dns.prefetch()
created: 2026-06-09
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# Bun.dns.prefetch()

Type: TOOL

## From [[drive-research-advanced-typescript-execution-in-the-bun-runtime-micro03|drive-research-advanced-typescript-execution-in-the-bun-runtime-micro03]] (2026-06-09)
- For extreme performance finesse, developers can utilize the experimental Bun.dns.prefetch(hostname, port) API.
- By preemptively issuing DNS queries during application startup—for instance, resolving a remote PostgreSQL database host while the application is parsing configuration files—the DNS resolution overhead is completely negated by the time the first database connection is initiated.
- Advanced DNS Control.
- Networking latency often stems from domain name resolution prior to TCP handshakes.
- Bun incorporates a robust internal DNS cache that persists for up to 30 seconds.

## From [[drive-research-bun-typescript-performance-tips-micro03|drive-research-bun-typescript-performance-tips-micro03]] (2026-06-09)
- Experimental API.
- Preemptively issues DNS queries during application startup.
