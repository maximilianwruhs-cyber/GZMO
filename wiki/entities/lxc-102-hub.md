---
type: entity
title: LXC 102 (Hub)
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# LXC 102 (Hub)

Type: SYSTEM

## From [[drive-research-proxmox-agent-data-storage-micro03|drive-research-proxmox-agent-data-storage-micro03]] (2026-06-09)
- Receives the payload from the AI Client.
- Verifies the client's Bearer token.
- Identifies the target tool as neo4j__get-schema.
- Maps the request to the namespaced database server running on LXC 101.
