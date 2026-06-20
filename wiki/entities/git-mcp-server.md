---
type: entity
title: Git MCP Server
created: 2026-06-09
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# Git MCP Server

Type: TOOL

## From [drive-research-proxmox-agent-data-storage-micro02](/entities/drive-research-proxmox-agent-data-storage-micro02.md) (2026-06-09)
- Runs in LXC 100 to expose programmatic Git repository manipulation.
- Allows LLM agents to track code modifications.

## From [drive-research-proxmox-agent-data-storage-micro03](/entities/drive-research-proxmox-agent-data-storage-micro03.md) (2026-06-09)
- Called by the client on LXC 100 following file-write confirmation.
- Programmatically commits the modification.
- A PyPI package for a Git MCP server.
