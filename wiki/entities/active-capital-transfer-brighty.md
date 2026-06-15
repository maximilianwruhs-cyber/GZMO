---
type: entity
title: Active Capital Transfer (Brighty)
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Active Capital Transfer (Brighty)

Type: TOOL

## From [[openclaw-autonomous-ai-agents-in-financial-operat|openclaw-autonomous-ai-agents-in-financial-operat]] (2026-06-08)
- A specialized financial skill for OpenClaw.
- Enables active deployment of capital and money transfers.
- Connects to Brighty MCP server via the mcporter tool.
- Skill used for SEPA transfers.
- Requires HITL approval for Level 4 operations.
- Skill used for Active Capital Transfer.
- Connects to the Brighty MCP server.
- Connects the brighty skill via the mcporter tool to the Brighty MCP server.
- Extracts SEPA IBANs from incoming invoices via OCR.
- Runs AML blacklist checks, formats payload, and queues international SEPA/SWIFT payment.
- High-value API credential.
- Must be stored securely in .env files or credential managers.
- Token used for Brighty MCP.
- Stored in .env file.
