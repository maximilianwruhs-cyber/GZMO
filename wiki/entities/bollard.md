---
type: entity
title: bollard
created: 2026-06-09
updated: 2026-06-10
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# bollard

Type: TOOL

## From [[the-cognitive-architecture-of-openclaw-agents-micro04|the-cognitive-architecture-of-openclaw-agents-micro04]] (2026-06-09)
- Docker API client used for OS-level sandboxing.
- Utilized by the openclaw-skills crate to spin up Docker containers.

## From [[resilient-rust-based-mcp-client-and-llm-orchestrat-micro02|resilient-rust-based-mcp-client-and-llm-orchestrat-micro02]] (2026-06-10)
- Asynchronous client library for interacting with Docker and Podman APIs.
- Provides a programmatic interface to container orchestration.
- Uses hyper for HTTP transactions and tokio for asynchronous runtime.
