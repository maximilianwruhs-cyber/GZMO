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
- [gray-matter parsing module](/entities/gray-matter-parsing-module.md) (TOOL)
- [Progressive Disclosure](/entities/progressive-disclosure.md) (CONCEPT)
- [Zod](/entities/zod.md) (TOOL)
- [nordwestt/ollama-ai-provider-v2](/entities/nordwestt-ollama-ai-provider-v2.md) (SYSTEM)
- [Vercel AI SDK](/entities/vercel-ai-sdk.md) (TOOL)
- [experimental_createSkillTool](/entities/experimental-createskilltool.md) (CONCEPT)
- [Scout Pattern](/entities/scout-pattern.md) (CONCEPT)
- [JSON Schema](/entities/json-schema.md) (CONCEPT)
- [dynamicTool()](/entities/dynamictool.md) (CONCEPT)
- [just-bash](/entities/just-bash.md) (TOOL)
- [bash-tool](/entities/bash-tool.md) (CONCEPT)
- [experimental_onToolCallFinish](/entities/experimental-ontoolcallfinish.md) (CONCEPT)
- [YAML frontmatter](/entities/yaml-frontmatter.md) (CONCEPT)
- [SKILL.md](/entities/skill-md.md) (CONCEPT)
- [Markdown body](/entities/markdown-body.md) (CONCEPT)
- [OpenClaw](/entities/openclaw.md) (SYSTEM)
- [maxSteps](/entities/maxsteps.md) (CONCEPT)
- [remark-parse](/entities/remark-parse.md) (TOOL)
- [front-matter](/entities/front-matter.md) (TOOL)
- [experimental_repairToolCall](/entities/experimental-repairtoolcall.md) (CONCEPT)
- [NoSuchToolError](/entities/nosuchtoolerror.md) (CONCEPT)
- [@vercel/sandbox](/entities/vercel-sandbox.md) (TOOL)
- [InvalidToolArgumentsError](/entities/invalidtoolargumentserror.md) (CONCEPT)
- [experimental_onToolCallStart](/entities/experimental-ontoolcallstart.md) (CONCEPT)

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
