---
type: entity
title: IP Helper (Iphlpapi)
created: 2026-06-08
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# IP Helper (Iphlpapi)

Type: SYSTEM

## From [[architectural-strategy-for-stealthy-portable-cli-a|architectural-strategy-for-stealthy-portable-cli-a]] (2026-06-08)
- Interface on Windows.
- Contains the GetExtendedTcpTable API.
- Used to get a snapshot of all active TCP connections and listening ports.

## From [[drive-research-architecting-zero-configuration-portable-agents-s-micro03|drive-research-architecting-zero-configuration-portable-agents-s-micro03]] (2026-06-09)
- Windows interface from which GetExtendedTcpTable API is invoked.
- Returns a snapshot of active TCP connections and listening ports.
