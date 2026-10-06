---
title: OpenClaw Agents Configuration
---

## Default Settings

Operator surface for GZMO via Telegram OpenClaw.

### Security

- No direct Qdrant upserts
- No Neo4j auto-graph from chat
- No second overnight writer
- Knowledge transfer via takeaway enqueue only

### Integration

1. `gzmo-living` MCP · `gzmo_memory_search`
2. `bin/openclaw-takeaway.sh`
3. Local inference: see `MODELS.md` (vLLM dual-Blackwell)
