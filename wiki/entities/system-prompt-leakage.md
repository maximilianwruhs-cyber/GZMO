---
type: entity
title: System Prompt Leakage
created: 2026-06-09
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---



# System Prompt Leakage

Type: CONCEPT

## From [[gzmo-soul-merged-new-part1|gzmo-soul-merged-new-part1]] (2026-06-09)
- Ist eine fundamentale konzeptionelle Schwachstelle
- Wird als OWASP LLM07 bezeichnet

## From [[drive-research-architektur-und-implementierung-intelligenter-ki-a-micro02|drive-research-architektur-und-implementierung-intelligenter-ki-a-micro02]] (2026-06-09)
- Functions as the agent's operating system.
- Defines persona, behavior rules, tool usage, and goals.
- Precise formulation is crucial for effective agent operation.
- Can contain sensitive information leading to 'System Prompt Leakage'.
- Secrets should not be hardcoded directly in the text.
- Functions as a 'Shared Source of Truth' between operator and AI.
- The initial input for the judge LLM in an evaluation.
- Defines the agent's behavior and goals.
- A fundamental conceptual vulnerability in autonomous agent design (OWASP LLM07).
- Occurs when system prompts contain sensitive information that can be extracted.
