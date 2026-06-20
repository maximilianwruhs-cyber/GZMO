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
- [hyper](/entities/hyper.md) (TOOL)
- [LogOutput](/entities/logoutput.md) (CONCEPT)
- [HostConfig](/entities/hostconfig.md) (CONCEPT)
- [containerd](/entities/containerd.md) (SYSTEM)
- [runsc](/entities/runsc.md) (TOOL)
- [Moby project](/entities/moby-project.md) (ORGANIZATION)
- [Rust](/entities/rust.md) (CONCEPT)
- [Podman](/entities/podman.md) (SYSTEM)
- [Bollard](/entities/bollard.md) (TOOL)
- [Config](/entities/config.md) (CONCEPT)
- [Docker](/entities/docker.md) (SYSTEM)
- [gVisor](/entities/gvisor.md) (SYSTEM)
- [tokio](/entities/tokio.md) (TOOL)
- [cgroups](/entities/cgroups.md) (CONCEPT)

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
