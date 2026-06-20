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
- [Google Workspace Extension](/entities/google-workspace-extension.md) (TOOL)
- [gas-fakes auth](/entities/gas-fakes-auth.md) (TOOL)
- [Antigravity](/entities/antigravity.md) (SYSTEM)
- [Manage MCP Servers](/entities/manage-mcp-servers.md) (CONCEPT)
- [IDE](/entities/ide.md) (SYSTEM)
- [mcp_config.json](/entities/mcp-config-json.md) (TOOL)
- [gemini CLI](/entities/gemini-cli.md) (TOOL)
- [proxy script](/entities/proxy-script.md) (TOOL)
- [Playground](/entities/playground.md) (SYSTEM)
- [Agent Manager](/entities/agent-manager.md) (SYSTEM)
- [Google Drive](/entities/google-drive.md) (SYSTEM)
- [MCP server](/entities/mcp-server.md) (SYSTEM)

## Relations
- gemini CLI → USES → Google Workspace Extension
- Antigravity → USES → Agent Manager
- Agent Manager → USES → Manage MCP Servers
- MCP server → PART_OF → mcp_config.json
- gas-fakes auth → USES → Google Workspace Extension
- Antigravity → USES → Playground
- Antigravity → USES → IDE
- Google Drive → RELATED_TO → Google Workspace Extension
