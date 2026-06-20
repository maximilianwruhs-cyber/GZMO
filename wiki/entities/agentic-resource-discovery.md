---
type: entity
title: "Agentic Resource Discovery"
created: "2026-06-20"
updated: "2026-06-20"
status: draft
sources: 1
tags:
  - research
  - ard
---

# Agentic Resource Discovery

Agentic Resource Discovery (ARD) describes how autonomous agents locate, evaluate, and consume heterogeneous resources without central registries.

## GZMO mapping (Scan → Identify → Map → Monitor)

| ARD phase | GZMO mechanism |
|-----------|----------------|
| Scan | `gzmo_health`, MCP `list`, `probe-ard-mcp-surface.sh` |
| Identify | Skill registry, MCP tool bridges |
| Map | Obolus engine routing, pillar B/C dispatch |
| Monitor | PulseLoop tension, Obolus ledger, synapse events |

## Pillar assignment

- **B** — execution and tool dispatch
- **C** — MCP surface and infra probes
- **S** — honeypot as semantic resource + security validation

VCG auctions and P2P barter are **explicitly out of scope** under Sovereign Node Directive.
