---
type: entity
title: "Network Exception Tier"
created: "2026-06-20"
updated: "2026-06-20"
status: draft
sources: 1
tags:
  - compliance
  - snd
  - research
---

# Network Exception Tier

GZMO Sovereign Node compliance uses three tiers. The directive is intentionally strict for Tier 1; Tier 2 is a permanent outbound exception for research retrieval.

## Tier 1 — Core SND (strict)

- Prime `:8000`, vault, honeypot, Neo4j, Qdrant, episodic memory
- LAN `192.168.31.0/24` only for persistence and cognition
- No cloud inference by default

## Tier 2 — Network exceptions (always allowed)

Configured in `gzmo.toml` → `[compliance].network_exceptions`:

| Exception | Capability |
|-----------|------------|
| `web_search` | `web_search` / `web_read` tools, `web_search.sh` |
| `agent-reach` | Agent-Reach CLIs, ASH, `~/.agent-reach/` sandboxes |
| `arxiv` | OAI-PMH, export.arxiv.org API, `skill_arxiv.sh`, Semantic Scholar graph |

Implementation: `gzmo-core/src/compliance.rs`

## Tier 3 — Cloud opt-in

- `allow_cloud_engine`, `allow_cloud_tools`, SerpAPI
- Requires explicit operator configuration

## Data rule

Tier 2 permits outbound retrieval; promoted facts still land in local vault/honeypot only.
