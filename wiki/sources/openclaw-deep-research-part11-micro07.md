---
type: source
title: openclaw-deep-research-part11-micro07
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# openclaw-deep-research-part11-micro07

Ingested source summary (2026-06-09).

## Entities
- [[openprose|OpenProse]] (TOOL)
- [[email-triage|Email triage]] (CONCEPT)
- [[workflow-files-lobster|Workflow files (.lobster)]] (TOOL)
- [[openclaw|OpenClaw]] (SYSTEM)
- [[llm-task|LLM Task]] (TOOL)
- [[brain-cli|brain-cli]] (TOOL)

## Relations
- Workflow files (.lobster) → USES → LLM Task
- OpenClaw → USES → Workflow files (.lobster)
- OpenClaw → USES → LLM Task
- Workflow files (.lobster) → RELATED_TO → OpenProse
- OpenProse → USES → Workflow files (.lobster)
- Workflow files (.lobster) → RELATED_TO → Email triage
- brain-cli → USES → Workflow files (.lobster)
- brain-cli → USES → OpenClaw
