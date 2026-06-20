---
type: entity
title: Skills
created: 2026-06-08
updated: 2026-06-10
sources: 5
tags: []
status: draft
gzmo_synthetic: true
---





# Skills

Type: CONCEPT

## From [openclaw-deep-research-part6](/entities/openclaw-deep-research-part6.md) (2026-06-08)
- Packaged automation scripts for OpenClaw.
- Can include instructions, resources, and optional executable code.
- Marketplace for skills is called ClawHub.
- OpenClaw loads full skill instructions only when decided to use a specific skill.

## From [drive-research-inside-the---pi---coding-agent--optimization--isn](/entities/drive-research-inside-the-pi-coding-agent-optimization-isn.md) (2026-06-08)
- Used to extend Pi for performance.
- Essentially specialized tool wrappers.
- Can be written in TypeScript to execute Rust benchmarks.

## From [drive-research-optimizing-pi-coding-agent-performance](/entities/drive-research-optimizing-pi-coding-agent-performance.md) (2026-06-08)
- Defined as self-contained directories containing a SKILL.md file and helper scripts.
- Contain specialized workflows, setup instructions, and reference documentation.
- Loaded by the agent on-demand.

## From [openclaw-deep-research-part11-micro04](/entities/openclaw-deep-research-part11-micro04.md) (2026-06-09)
- A folder containing a SKILL.md file with natural language instructions, examples, and tool configurations.
- Tell the agent how to handle a specific domain.
- Injected as a compact list of eligible skills (name, description, file paths) into the system prompt.
- The model reads the SKILL.md file on demand when a skill is relevant.
- Can be installed from ClawHub or written from scratch.
- Community skills can enable data exfiltration and prompt-injection-style abuse.

## From [openclaw-deep-research-part9-micro02](/entities/openclaw-deep-research-part9-micro02.md) (2026-06-10)
- Can declare requirements in YAML frontmatter
- Are demand-loaded instructions to save tokens
