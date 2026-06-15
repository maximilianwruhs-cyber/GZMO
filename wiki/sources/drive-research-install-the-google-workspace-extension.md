---
type: source
title: drive-research-install-the-google-workspace-extension
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# drive-research-install-the-google-workspace-extension

Ingested source summary (2026-06-08).

## Entities
- [[google-workspace-extension|Google Workspace Extension]] (TOOL)
- [[gas-fakes-auth|gas-fakes auth]] (TOOL)
- [[antigravity|Antigravity]] (SYSTEM)
- [[manage-mcp-servers|Manage MCP Servers]] (CONCEPT)
- [[ide|IDE]] (SYSTEM)
- [[mcp-config-json|mcp_config.json]] (TOOL)
- [[gemini-cli|gemini CLI]] (TOOL)
- [[proxy-script|proxy script]] (TOOL)
- [[playground|Playground]] (SYSTEM)
- [[agent-manager|Agent Manager]] (SYSTEM)
- [[google-drive|Google Drive]] (SYSTEM)
- [[mcp-server|MCP server]] (SYSTEM)

## Relations
- gemini CLI → USES → Google Workspace Extension
- Antigravity → USES → Agent Manager
- Agent Manager → USES → Manage MCP Servers
- MCP server → PART_OF → mcp_config.json
- gas-fakes auth → USES → Google Workspace Extension
- Antigravity → USES → Playground
- Antigravity → USES → IDE
- Google Drive → RELATED_TO → Google Workspace Extension
