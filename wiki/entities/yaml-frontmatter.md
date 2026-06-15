---
type: entity
title: YAML frontmatter
created: 2026-06-08
updated: 2026-06-08
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# YAML frontmatter

Type: CONCEPT

## From [[local-first-rag-architecting-sovereign-ai-with-li|local-first-rag-architecting-sovereign-ai-with-li]] (2026-06-08)
- Metadata is universally managed via YAML frontmatter blocks located at the absolute top of the Markdown file in an Obsidian vault.
- The bridging script must programmatically construct and inject a robust YAML schema.
- Tags within the YAML array must not include the # prefix.

## From [[migrating-openclaw-to-vercel-ai-sdk-for-local-llm|migrating-openclaw-to-vercel-ai-sdk-for-local-llm]] (2026-06-08)
- A component of the SKILL.md file.
- Serves as the critical routing, gating, and dependency injection layer for the skill.
- Contains metadata like 'name', 'description', 'metadata.openclaw.requires.env', 'metadata.openclaw.requires.bins', 'user-invocable', and 'context'.
