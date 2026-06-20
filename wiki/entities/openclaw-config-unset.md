---
type: entity
title: openclaw config unset
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# openclaw config unset

Type: TOOL

## From [openclaw-deep-research-part1-micro03](/entities/openclaw-deep-research-part1-micro03.md) (2026-06-09)
- Unsets a specific configuration value.
- Example: openclaw config unset plugins.entries.brave.config.webSearch.apiKey
- Reads an optional JSON5 config from ~/.openclaw/openclaw.json.
- Uses safe defaults if the config file is missing.
- Can be configured to connect channels, set models, tools, sandboxing, or automation.
- Gateway watches the config file and applies changes automatically.
