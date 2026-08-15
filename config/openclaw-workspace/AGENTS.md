---
title: Oh My Pi Agents Configuration
---

## Oh My Pi Agent Configuration

### Default Settings
This agent is configured to work within the GZMO ecosystem with the following default settings:

#### Skills
- `character`: Enables persona selection via Telegram commands
- All other skills are configured according to GZMO security policies

#### Security
- Operates under the same security constraints as other OpenClaw agents
- No direct Qdrant upserts
- No Neo4j auto-graph from chat
- No second overnight writer
- All knowledge transfer uses the takeaway enqueue mechanism

### Integration with GZMO Living Memory
This agent connects to the GZMO living memory system through:
1. `gzmo-living` MCP server
2. `gzmo_memory_search` for knowledge retrieval
3. `openclaw-takeaway.sh` for durable fact intake

### Telegram Integration
The agent responds to Telegram commands with the following capabilities:
- `/character list` - Lists available personas
- `/character who` - Shows current persona
- `/character glados` - Selects the GLaDOS persona
- `/character search duck` - Searches for personas matching "duck"

### Configuration Notes
This agent is designed to be a secure operator surface that:
1. Maintains proper connection to the CT101 living memory system
2. Follows all security protocols
3. Operates within the defined GZMO ecosystem boundaries