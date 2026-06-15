---
type: source
title: resilient-rust-based-mcp-client-and-llm-orchestrat-micro02
created: 2026-06-10
updated: 2026-06-10
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# resilient-rust-based-mcp-client-and-llm-orchestrat-micro02

Ingested source summary (2026-06-10).

## Entities
- [[hyper|hyper]] (TOOL)
- [[logoutput|LogOutput]] (CONCEPT)
- [[hostconfig|HostConfig]] (CONCEPT)
- [[containerd|containerd]] (SYSTEM)
- [[runsc|runsc]] (TOOL)
- [[moby-project|Moby project]] (ORGANIZATION)
- [[rust|Rust]] (CONCEPT)
- [[podman|Podman]] (SYSTEM)
- [[bollard|Bollard]] (TOOL)
- [[config|Config]] (CONCEPT)
- [[docker|Docker]] (SYSTEM)
- [[gvisor|gVisor]] (SYSTEM)
- [[tokio|tokio]] (TOOL)
- [[cgroups|cgroups]] (CONCEPT)

## Relations
- Bollard → USES → Docker
- Bollard → USES → Podman
- Bollard → USES → tokio
- Bollard → USES → hyper
- Bollard → USES → LogOutput
- Bollard → USES → Config
- Bollard → USES → HostConfig
- Docker → USES → containerd
- containerd → USES → gVisor
- Bollard → RELATED_TO → Moby project
