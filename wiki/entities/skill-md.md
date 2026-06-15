---
type: entity
title: SKILL.md
created: 2026-06-08
updated: 2026-06-10
sources: 17
tags: []
status: draft
gzmo_synthetic: true
---

















# SKILL.md

Type: CONCEPT

## From [[ai-research-part7|ai-research-part7]] (2026-06-08)
- File containing YAML frontmatter.
- Part of the structural composition of Agent Skills.

## From [[migrating-openclaw-to-vercel-ai-sdk-for-local-llm|migrating-openclaw-to-vercel-ai-sdk-for-local-llm]] (2026-06-08)
- A declarative, human-readable format used by OpenClaw.
- Acts as a self-contained unit of capability, packaging procedural knowledge, execution context, and environmental requirements.
- The root of an OpenClaw skill's directory structure, serving as the entry point, manifest, and primary instruction manual.

## From [[openclaw-deep-research-part4|openclaw-deep-research-part4]] (2026-06-08)
- A file that forms the core of OpenClaw skills.
- Contains natural-language instructions for the AI.
- Tells the AI how to perform a specific task.

## From [[drive-research-deep-dive-google-antigravity-architecture|drive-research-deep-dive-google-antigravity-architecture]] (2026-06-08)
- A knowledge base framework.
- Skills are encapsulated within a SKILL.md file containing YAML frontmatter.

## From [[drive-research-deep-dive-google-antigravity-architecture1|drive-research-deep-dive-google-antigravity-architecture1]] (2026-06-08)
- A knowledge base framework.
- Each individual Skill is encapsulated within a SKILL.md file.
- Contains YAML frontmatter defining the skill's name and description.
- Followed by detailed operational instructions, stylistic conventions, and actionable scripts.

## From [[drive-research-hermes-agent-prompt-builder-analysis|drive-research-hermes-agent-prompt-builder-analysis]] (2026-06-08)
- Contains YAML frontmatter for installed capabilities.
- The _parse_skill_file function is artificially constrained to read only the first 2,000 characters of this file.
- Local files whose metadata bypasses security scanning.
- The description field of YAML frontmatter is directly ingested.

## From [[drive-research-pi-coding-agent-ecosystem-tier-list|drive-research-pi-coding-agent-ecosystem-tier-list]] (2026-06-08)
- Contains a YAML frontmatter header defining the skill's name and description.
- The full body is dynamically loaded into the active context when the skill is triggered.
- Can package sub-folders such as scripts/, references/, and assets/.

## From [[drive-research-pi-coding-agent-local-deployment-customization|drive-research-pi-coding-agent-local-deployment-customization]] (2026-06-08)
- Mandatory file in a skill package containing YAML frontmatter and markdown instructions.
- Defines metadata used by the agent during the discovery phase.

## From [[gzmo|gzmo]] (2026-06-09)
- Plugin architecture file in OpenClaw for defining agent skills
- GZMO uses to self-extend capabilities when lacking tools

## From [[openclaw-deep-research-part1-micro01|openclaw-deep-research-part1-micro01]] (2026-06-09)
- The SKILL.md file must include name and description
- Skill instructions for Codex to follow

## From [[the-cognitive-architecture-of-openclaw-agents-micro02|the-cognitive-architecture-of-openclaw-agents-micro02]] (2026-06-09)
- Contains the modular execution instructions required to perform actual computational work.
- Employs a rigorous 'Lazy Loading' architecture.
- Skills are compartmentalized based on specific library or framework dependencies.

## From [[the-cognitive-architecture-of-openclaw-agents-micro03|the-cognitive-architecture-of-openclaw-agents-micro03]] (2026-06-09)
- Shell code can be executed via a loaded SKILL.md.
- Represents compartmentalized, dynamic utility.

## From [[the-cognitive-architecture-of-openclaw-agents-micro04|the-cognitive-architecture-of-openclaw-agents-micro04]] (2026-06-09)
- Associated with the openclaw-skills crate.
- Involves lazy-loading and .skillsrc parsing.
- Can be injected as an ephemeral message struct with is_meta: true flag.

## From [[the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro02|the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro02]] (2026-06-09)
- Files used to store AgentSkills with YAML frontmatter.

## From [[the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro05|the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro05]] (2026-06-09)
- Contains 'requires.bins' which can be checked if an agent fails to invoke a tool.

## From [[gzmo-soul-merged-new-part2-micro08|gzmo-soul-merged-new-part2-micro08]] (2026-06-10)
- Verzeichnis für ausgelagerte Werkzeuge/Skills

## From [[openclaw-part1-micro05|openclaw-part1-micro05]] (2026-06-10)
- Enables 'Lazy Loading' of cognitive and operative capabilities.
- Uses a modular structure with YAML frontmatter and Markdown body.
- Prevents context window expansion and high API costs.
