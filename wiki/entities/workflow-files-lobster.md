---
type: entity
title: Workflow files (.lobster)
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Workflow files (.lobster)

Type: TOOL

## From [[openclaw-deep-research-part11-micro07|openclaw-deep-research-part11-micro07]] (2026-06-09)
- A workflow shell that lets OpenClaw run multi-step tool sequences.
- Provides deterministic pipelines, explicit approvals, and resumable state.
- Is an authoring layer above detached background work.
- Can run YAML/JSON workflow files with name, args, steps, env, condition, and approval fields.
- Is an optional plugin tool and not enabled by default.
- Can be YAML/JSON files defining Lobster workflows.
- Contain fields like name, args, steps, env, condition, and approval.
- Can be specified as the 'pipeline' in OpenClaw tool calls.
