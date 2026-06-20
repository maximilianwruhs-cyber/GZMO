---
type: entity
title: MIB_TCPTABLE_OWNER_PID
created: 2026-06-08
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# MIB_TCPTABLE_OWNER_PID

Type: CONCEPT

## From [architectural-strategy-for-stealthy-portable-cli-a](/entities/architectural-strategy-for-stealthy-portable-cli-a.md) (2026-06-08)
- Structure returned by GetExtendedTcpTable API.
- Contains a comprehensive snapshot of TCP connections and listening ports.
- Allows parsing to verify if a port is in a LISTEN state.

## From [drive-research-architecting-zero-configuration-portable-agents-s-micro03](/entities/drive-research-architecting-zero-configuration-portable-agents-s-micro03.md) (2026-06-09)
- Structure into which GetExtendedTcpTable maps active TCP connections and listening ports.
- Used for parsing memory tables to identify listening endpoints.
