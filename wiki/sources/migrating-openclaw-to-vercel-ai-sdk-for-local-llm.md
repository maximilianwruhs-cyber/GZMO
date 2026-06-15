---
type: source
title: migrating-openclaw-to-vercel-ai-sdk-for-local-llm
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# migrating-openclaw-to-vercel-ai-sdk-for-local-llm

Ingested source summary (2026-06-08).

## Entities
- [[gray-matter-parsing-module|gray-matter parsing module]] (TOOL)
- [[progressive-disclosure|Progressive Disclosure]] (CONCEPT)
- [[zod|Zod]] (TOOL)
- [[nordwestt-ollama-ai-provider-v2|nordwestt/ollama-ai-provider-v2]] (SYSTEM)
- [[vercel-ai-sdk|Vercel AI SDK]] (TOOL)
- [[experimental-createskilltool|experimental_createSkillTool]] (CONCEPT)
- [[scout-pattern|Scout Pattern]] (CONCEPT)
- [[json-schema|JSON Schema]] (CONCEPT)
- [[dynamictool|dynamicTool()]] (CONCEPT)
- [[just-bash|just-bash]] (TOOL)
- [[bash-tool|bash-tool]] (CONCEPT)
- [[experimental-ontoolcallfinish|experimental_onToolCallFinish]] (CONCEPT)
- [[yaml-frontmatter|YAML frontmatter]] (CONCEPT)
- [[skill-md|SKILL.md]] (CONCEPT)
- [[markdown-body|Markdown body]] (CONCEPT)
- [[openclaw|OpenClaw]] (SYSTEM)
- [[maxsteps|maxSteps]] (CONCEPT)
- [[remark-parse|remark-parse]] (TOOL)
- [[front-matter|front-matter]] (TOOL)
- [[experimental-repairtoolcall|experimental_repairToolCall]] (CONCEPT)
- [[nosuchtoolerror|NoSuchToolError]] (CONCEPT)
- [[vercel-sandbox|@vercel/sandbox]] (TOOL)
- [[invalidtoolargumentserror|InvalidToolArgumentsError]] (CONCEPT)
- [[experimental-ontoolcallstart|experimental_onToolCallStart]] (CONCEPT)

## Relations
- OpenClaw → RELATED_TO → Vercel AI SDK
- OpenClaw → USES → SKILL.md
- SKILL.md → PART_OF → YAML frontmatter
- SKILL.md → PART_OF → Markdown body
- OpenClaw → USES → Progressive Disclosure
- Vercel AI SDK → USES → bash-tool
- Vercel AI SDK → USES → dynamicTool()
- dynamicTool() → USES → JSON Schema
- bash-tool → USES → just-bash
- bash-tool → USES → experimental_createSkillTool
- bash-tool → RELATED_TO → @vercel/sandbox
- experimental_createSkillTool → USES → SKILL.md
- Vercel AI SDK → USES → maxSteps
- Vercel AI SDK → USES → experimental_onToolCallStart
- Vercel AI SDK → USES → experimental_onToolCallFinish
- Vercel AI SDK → USES → experimental_repairToolCall
- Scout Pattern → RELATED_TO → Progressive Disclosure
- gray-matter parsing module → USES → dynamicTool()
- experimental_repairToolCall → USES → NoSuchToolError
- experimental_repairToolCall → USES → InvalidToolArgumentsError
- bash-tool → RELATED_TO → OpenClaw
