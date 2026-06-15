---
type: entity
title: analysiere_letztes_bild
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# analysiere_letztes_bild

Type: CONCEPT

## From [[prompt-agent-engineering-part3|prompt-agent-engineering-part3]] (2026-06-08)
- A tool defined for the AI to use.
- Used when the user asks a question about a previously sent image.
- Called by the Python script to execute LLaVA.
- A function designed to be used by Llama 3 to control LLaVA.
- Takes a 'frage' (question) as a parameter.
- Requires the last image path to be stored in chat_sessions.
