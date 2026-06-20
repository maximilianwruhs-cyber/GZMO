---
type: entity
title: AETHER-LINK
created: 2026-06-09
updated: 2026-06-10
sources: 4
tags: []
status: draft
gzmo_synthetic: true
---




# AETHER-LINK

Type: CONCEPT

## From [aether-grid-micro02](/entities/aether-grid-micro02.md) (2026-06-09)
- gRPC/Protobuf over WireGuard + Vault-mTLS
- LRU-Cache-Miss -> Core-Request (<800 ms Level-2)

## From [prompt-agent-engineering-part4-micro02](/entities/prompt-agent-engineering-part4-micro02.md) (2026-06-10)
- The handshake/connection between Edge and Core

## From [prompt-agent-engineering-part4-micro03](/entities/prompt-agent-engineering-part4-micro03.md) (2026-06-10)
- The secure connection between the Edge and the Core
- Uses gRPC over a quantum-resistant WireGuard tunnel
- Employs Protobuf for binary serialization

## From [prompt-agent-engineering-part4-micro06](/entities/prompt-agent-engineering-part4-micro06.md) (2026-06-10)
- Protocol layer for mTLS/MQTT or REST execution
