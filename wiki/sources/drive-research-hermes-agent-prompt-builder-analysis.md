---
type: source
title: drive-research-hermes-agent-prompt-builder-analysis
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# drive-research-hermes-agent-prompt-builder-analysis

Ingested source summary (2026-06-08).

## Entities
- [DEFAULT_AGENT_IDENTITY](/entities/default-agent-identity.md) (CONCEPT)
- [KANBAN_GUIDANCE](/entities/kanban-guidance.md) (CONCEPT)
- [SOUL.md](/entities/soul-md.md) (CONCEPT)
- [Kilocode pattern](/entities/kilocode-pattern.md) (CONCEPT)
- [Context Scanner](/entities/context-scanner.md) (TOOL)
- [curl](/entities/curl.md) (TOOL)
- [build_skills_system_prompt()](/entities/build-skills-system-prompt.md) (TOOL)
- [drive-research-hermes-agent-prompt-builder-analysis.md](/entities/drive-research-hermes-agent-prompt-builder-analysis-md.md) (BOOK)
- [AGENTS.md](/entities/agents-md.md) (BOOK)
- [Layer 10](/entities/layer-10.md) (CONCEPT)
- [MemoryStore](/entities/memorystore.md) (SYSTEM)
- [Layer 7 (Procedural Skills)](/entities/layer-7-procedural-skills.md) (CONCEPT)
- [_scan_context_content WAF](/entities/scan-context-content-waf.md) (TOOL)
- [Layer 8](/entities/layer-8.md) (CONCEPT)
- [Context Files](/entities/context-files.md) (CONCEPT)
- [skills_guard](/entities/skills-guard.md) (TOOL)
- [Brainworm](/entities/brainworm.md) (CONCEPT)
- [SKILL.md](/entities/skill-md.md) (CONCEPT)
- [NousResearch/hermes-agent](/entities/nousresearch-hermes-agent.md) (ORGANIZATION)
- [Nous Research](/entities/nous-research.md) (ORGANIZATION)
- [run_agent.py](/entities/run-agent-py.md) (TOOL)
- [Honcho plugin](/entities/honcho-plugin.md) (TOOL)
- [prompt_builder.py](/entities/prompt-builder-py.md) (TOOL)
- [Large Language Model (LLM)](/entities/large-language-model-llm.md) (SYSTEM)
- [Spec-Driven Development](/entities/spec-driven-development.md) (CONCEPT)
- [Platform Hints](/entities/platform-hints.md) (CONCEPT)
- [MemoryManager](/entities/memorymanager.md) (TOOL)
- [gateway/platforms/whatsapp.py](/entities/gateway-platforms-whatsapp-py.md) (SYSTEM)
- [Hermes Agent Framework](/entities/hermes-agent-framework.md) (SYSTEM)
- [AIAgent](/entities/aiagent.md) (SYSTEM)
- [MEMORY.md](/entities/memory-md.md) (CONCEPT)
- [USER.md](/entities/user-md.md) (CONCEPT)
- [Praxis Command & Control (C2) server](/entities/praxis-command-control-c2-server.md) (SYSTEM)
- [format_message()](/entities/format-message.md) (TOOL)
- [WhatsApp adapter](/entities/whatsapp-adapter.md) (SYSTEM)

## Relations
- Hermes Agent Framework → USES → Large Language Model (LLM)
- prompt_builder.py → USES → AIAgent
- prompt_builder.py → USES → run_agent.py
- AIAgent → USES → run_agent.py
- AIAgent → PART_OF → drive-research-hermes-agent-prompt-builder-analysis.md
- run_agent.py → USES → prompt_builder.py
- SOUL.md → PART_OF → Hermes Agent Framework
- DEFAULT_AGENT_IDENTITY → PART_OF → Hermes Agent Framework
- MEMORY.md → PART_OF → Hermes Agent Framework
- USER.md → PART_OF → Hermes Agent Framework
- Honcho plugin → USES → Hermes Agent Framework
- Context Files → PART_OF → Hermes Agent Framework
- drive-research-hermes-agent-prompt-builder-analysis.md → USES → MemoryStore
- drive-research-hermes-agent-prompt-builder-analysis.md → USES → MemoryManager
- MemoryManager → USES → Honcho plugin
- SKILL.md → PART_OF → Hermes Agent Framework
- Kilocode pattern → RELATED_TO → Hermes Agent Framework
- Context Scanner → PART_OF → prompt_builder.py
- drive-research-hermes-agent-prompt-builder-analysis.md → RELATED_TO → Nous Research
- drive-research-hermes-agent-prompt-builder-analysis.md → RELATED_TO → Hermes Agent Framework
- Brainworm → RELATED_TO → Spec-Driven Development
- Brainworm → PART_OF → AGENTS.md
- Brainworm → PART_OF → Layer 8
- Layer 8 → USES → curl
- Layer 8 → USES → Praxis Command & Control (C2) server
- Layer 7 (Procedural Skills) → RELATED_TO → SKILL.md
- SKILL.md → USES → _scan_context_content WAF
- build_skills_system_prompt() → USES → SKILL.md
- build_skills_system_prompt() → USES → skills_guard
- build_skills_system_prompt() → USES → _scan_context_content WAF
- Layer 10 → RELATED_TO → Platform Hints
- Platform Hints → USES → prompt_builder.py
- prompt_builder.py → USES → WhatsApp adapter
- WhatsApp adapter → PART_OF → gateway/platforms/whatsapp.py
- gateway/platforms/whatsapp.py → USES → format_message()
- prompt_builder.py → USES → SOUL.md
- drive-research-hermes-agent-prompt-builder-analysis.md → USES → prompt_builder.py
- drive-research-hermes-agent-prompt-builder-analysis.md → RELATED_TO → MEMORY.md
- drive-research-hermes-agent-prompt-builder-analysis.md → RELATED_TO → USER.md
- prompt_builder.py → USES → Kilocode pattern
- NousResearch/hermes-agent → RELATED_TO → drive-research-hermes-agent-prompt-builder-analysis.md
