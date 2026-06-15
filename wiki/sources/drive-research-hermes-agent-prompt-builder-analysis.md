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
- [[default-agent-identity|DEFAULT_AGENT_IDENTITY]] (CONCEPT)
- [[kanban-guidance|KANBAN_GUIDANCE]] (CONCEPT)
- [[soul-md|SOUL.md]] (CONCEPT)
- [[kilocode-pattern|Kilocode pattern]] (CONCEPT)
- [[context-scanner|Context Scanner]] (TOOL)
- [[curl|curl]] (TOOL)
- [[build-skills-system-prompt|build_skills_system_prompt()]] (TOOL)
- [[drive-research-hermes-agent-prompt-builder-analysis-md|drive-research-hermes-agent-prompt-builder-analysis.md]] (BOOK)
- [[agents-md|AGENTS.md]] (BOOK)
- [[layer-10|Layer 10]] (CONCEPT)
- [[memorystore|MemoryStore]] (SYSTEM)
- [[layer-7-procedural-skills|Layer 7 (Procedural Skills)]] (CONCEPT)
- [[scan-context-content-waf|_scan_context_content WAF]] (TOOL)
- [[layer-8|Layer 8]] (CONCEPT)
- [[context-files|Context Files]] (CONCEPT)
- [[skills-guard|skills_guard]] (TOOL)
- [[brainworm|Brainworm]] (CONCEPT)
- [[skill-md|SKILL.md]] (CONCEPT)
- [[nousresearch-hermes-agent|NousResearch/hermes-agent]] (ORGANIZATION)
- [[nous-research|Nous Research]] (ORGANIZATION)
- [[run-agent-py|run_agent.py]] (TOOL)
- [[honcho-plugin|Honcho plugin]] (TOOL)
- [[prompt-builder-py|prompt_builder.py]] (TOOL)
- [[large-language-model-llm|Large Language Model (LLM)]] (SYSTEM)
- [[spec-driven-development|Spec-Driven Development]] (CONCEPT)
- [[platform-hints|Platform Hints]] (CONCEPT)
- [[memorymanager|MemoryManager]] (TOOL)
- [[gateway-platforms-whatsapp-py|gateway/platforms/whatsapp.py]] (SYSTEM)
- [[hermes-agent-framework|Hermes Agent Framework]] (SYSTEM)
- [[aiagent|AIAgent]] (SYSTEM)
- [[memory-md|MEMORY.md]] (CONCEPT)
- [[user-md|USER.md]] (CONCEPT)
- [[praxis-command-control-c2-server|Praxis Command & Control (C2) server]] (SYSTEM)
- [[format-message|format_message()]] (TOOL)
- [[whatsapp-adapter|WhatsApp adapter]] (SYSTEM)

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
