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
- [OpenProse](/entities/openprose.md) (TOOL)
- [Email triage](/entities/email-triage.md) (CONCEPT)
- [Workflow files (.lobster)](/entities/workflow-files-lobster.md) (TOOL)
- [OpenClaw](/entities/openclaw.md) (SYSTEM)
- [LLM Task](/entities/llm-task.md) (TOOL)
- [brain-cli](/entities/brain-cli.md) (TOOL)

## Relations
- Workflow files (.lobster) → USES → LLM Task
- OpenClaw → USES → Workflow files (.lobster)
- OpenClaw → USES → LLM Task
- Workflow files (.lobster) → RELATED_TO → OpenProse
- OpenProse → USES → Workflow files (.lobster)
- Workflow files (.lobster) → RELATED_TO → Email triage
- brain-cli → USES → Workflow files (.lobster)
- brain-cli → USES → OpenClaw
