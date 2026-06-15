---
type: source
title: resilient-rust-based-mcp-client-and-llm-orchestrat-micro03
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# resilient-rust-based-mcp-client-and-llm-orchestrat-micro03

Ingested source summary (2026-06-09).

## Entities
- [[runsc|runsc]] (SYSTEM)
- [[docker|Docker]] (SYSTEM)
- [[tini|tini]] (SYSTEM)
- [[gvisor|gVisor]] (SYSTEM)
- [[bollard-api|Bollard API]] (TOOL)
- [[linux-virtual-file-system-vfs|Linux Virtual File System (VFS)]] (SYSTEM)
- [[logsoptions|LogsOptions]] (SYSTEM)
- [[tokio|Tokio]] (SYSTEM)
- [[alpine-latest|alpine:latest]] (SYSTEM)
- [[hostconfig|HostConfig]] (SYSTEM)
- [[read-only-bind-mount|read-only bind mount]] (CONCEPT)
- [[config|Config]] (SYSTEM)
- [[createcontaineroptions|CreateContainerOptions]] (SYSTEM)
- [[ephemeral-container-pattern|ephemeral container pattern]] (CONCEPT)
- [[logoutput|LogOutput]] (SYSTEM)
- [[waitcontaineroptions|WaitContainerOptions]] (SYSTEM)
- [[oci-image|OCI image]] (CONCEPT)
- [[selinux|SELinux]] (CONCEPT)

## Relations
- Bollard API → USES → HostConfig
- Bollard API → USES → Docker
- tini → PART_OF → HostConfig
- Linux Virtual File System (VFS) → RELATED_TO → read-only bind mount
- gVisor → USES → runsc
- runsc → PART_OF → gVisor
- Tokio → USES → Docker
- HostConfig → USES → Bollard API
- Config → USES → HostConfig
- Config → USES → alpine:latest
- Docker → USES → CreateContainerOptions
- Docker → USES → Config
- Docker → USES → LogsOptions
- Docker → USES → LogOutput
- LogOutput → USES → Docker
