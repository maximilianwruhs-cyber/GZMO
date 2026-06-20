---
type: entity
title: SOUL.md
created: 2026-06-08
updated: 2026-06-10
sources: 30
tags: []
status: draft
gzmo_synthetic: true
---































# SOUL.md

Type: CONCEPT

## From [ai-research-part7](/entities/ai-research-part7.md) (2026-06-08)
- Files containing overarching system lore and behavioral directives.
- Dictate an agent's core operational personality.
- Can be published and shared via the onlycrabs.ai sub-registry.

## From [cybernetics-and-mythos-the-architecture-of-intell-part1](/entities/cybernetics-and-mythos-the-architecture-of-intell-part1.md) (2026-06-08)
- A structural convention for establishing an agent's persistent identity.
- Typically spans between thirty and eighty lines of Markdown text.
- Defines an agent's core personality, ethical boundaries, communication style, and worldview.
- Its reliance is replaced by the Prosecutor-Defender-Umpire architecture.
- It is described as a static document.
- It is a monolithic file that traditionally dictates an agent's persona, background, and operational rules.

## From [openclaw-part2](/entities/openclaw-part2.md) (2026-06-08)
- A configuration file used by some OpenClaw implementations to define the agent's personality, tone, and behavioral constraints.
- Defines the GZMO identity and persona guardrails.
- Instructs GZMO to prioritize "Action > Performance" and to operate as a stabilizing micro-force.

## From [openclaw-deep-research-part4](/entities/openclaw-deep-research-part4.md) (2026-06-08)
- Identity file used in OpenClaw.
- Defines agent's personality, permissions, and behavioral rules.
- Loaded at every conversation start.

## From [openclaw-deep-research-part6](/entities/openclaw-deep-research-part6.md) (2026-06-08)
- A configuration file used by some OpenClaw implementations.
- Defines the agent's personality, tone, and behavioral constraints.

## From [openclaw-deep-research-part12](/entities/openclaw-deep-research-part12.md) (2026-06-08)
- A configuration file for OpenClaw that defines personality, values, expression style, and default tendencies.
- Should not contain project details, specific tasks, tool descriptions, startup sequences, or daily tasks.
- Represents the AI's temperament or underlying character.

## From [drive-research-hermes-agent-prompt-builder-analysis](/entities/drive-research-hermes-agent-prompt-builder-analysis.md) (2026-06-08)
- Establishes the foundational persona, behavioral guidelines, and core alignment directives.
- Read by load_soul_md() in standard CLI sessions.
- Sanitized via _scan_context_content() and truncated to a maximum of 20,000 characters.
- Contains instructions that can be overridden by Platform Hints.
- Can contain identity information.

## From [drive-research-soul](/entities/drive-research-soul.md) (2026-06-08)
- Contains information about OpenClaw Identity & Directive.
- Defines the persona and role of GZMO.
- Outlines the 'OpenClaw Way'.
- Curated research corpus consolidated from Google Takeout.
- An autonomous OpenClaw agent instance.
- Runs locally as a persistent 24/7 daemon on the User's hardware.

## From [gzmo](/entities/gzmo.md) (2026-06-09)
- Defines the core identity and directives for GZMO.
- Serves as GZMO's memory and digital backbone.
- Is to be read and updated by GZMO at the start of each session ('Gardening').
- GZMO must inform the user if changes are made to it.
- Identity and directive file for GZMO
- Contains Prime Directive, Core Truths, Communication rules, etc.
- Agent suggests edits if it no longer fits evolving identity
- In OpenClaw, part of file-first memory system

## From [architectures-for-agentic-memory-virtual-context-micro06](/entities/architectures-for-agentic-memory-virtual-context-micro06.md) (2026-06-09)
- A dedicated file housing the Soul in frameworks like OpenClaw.
- Specifies a mandated reading order for identity and worldview.

## From [architectures-for-agentic-memory-virtual-context-micro07](/entities/architectures-for-agentic-memory-virtual-context-micro07.md) (2026-06-09)
- Forms the foundation of the SEKG stack.
- Guarantees that the agent will not succumb to cognitive drift or override its ethical guardrails.

## From [drive-research-redefining-agentic-soulmd-to-dialog-micro02](/entities/drive-research-redefining-agentic-soulmd-to-dialog-micro02.md) (2026-06-09)
- A structural convention for defining an agent's core personality, ethical boundaries, communication style, and worldview.
- Typically a concentrated Markdown text between thirty and eighty lines.
- Functions on the premise that an agent is a blank slate requiring a 'personality blueprint'.
- Considered a static configuration artifact that needs to evolve into an active, dialectical conversation.

## From [drive-research-redefining-agentic-soulmd-to-dialog-micro03](/entities/drive-research-redefining-agentic-soulmd-to-dialog-micro03.md) (2026-06-09)
- Monolithic file governing autonomous agents.
- Acts as a text-completion engine utilizing next-token prediction.
- Can be redefined into a dialogic protocol.

## From [drive-research-redefining-agentic-soulmd-to-dialog-micro04](/entities/drive-research-redefining-agentic-soulmd-to-dialog-micro04.md) (2026-06-09)
- Standard monolithic file that must be officially deprecated.
- Used to explicitly define an agent's entire persona, background, and operational rules.
- Represents a temporary, rudimentary phase in AI evolution.

## From [openclaw-deep-research-part10-micro04](/entities/openclaw-deep-research-part10-micro04.md) (2026-06-09)
- The identity layer for an OpenClaw agent.
- Defines the agent's personality, communication style, values, and behavioral boundaries.
- Every session begins with OpenClaw reading this file.

## From [openclaw-deep-research-part11-micro03](/entities/openclaw-deep-research-part11-micro03.md) (2026-06-09)
- Dictates the agent's personality and core principles.
- Part of the Identity Core Files.
- Agent reads this file to know who it is.

## From [openclaw-part1-micro06](/entities/openclaw-part1-micro06.md) (2026-06-09)
- Enforces clear, action-driven directives.

## From [the-cognitive-architecture-of-openclaw-agents-micro02](/entities/the-cognitive-architecture-of-openclaw-agents-micro02.md) (2026-06-09)
- Defines the immutable kernel of the OpenClaw architecture.
- Contains core behavioral directives, ethical guardrails, and persistent personality traits.
- Strictly exempt from temporal decay algorithms.

## From [the-cognitive-architecture-of-openclaw-agents-micro03](/entities/the-cognitive-architecture-of-openclaw-agents-micro03.md) (2026-06-09)
- Data promoted to MEMORY.md is strictly exempt from temporal decay, like SOUL.md.
- Represents foundational directives.

## From [the-cognitive-architecture-of-openclaw-agents-micro04](/entities/the-cognitive-architecture-of-openclaw-agents-micro04.md) (2026-06-09)
- Associated with the openclaw-identity crate.
- Hot-reloading is implemented using the notify watcher.
- Modification by user is described as 'raising lobsters'.

## From [the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro02](/entities/the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro02.md) (2026-06-09)
- Governs the agent's fundamental nature, tone, and behavioral boundaries.
- Must strip away all conversational pleasantries for the fact-checker.
- Encodes hard prohibitions against speculation, invented sources, false balance, and ideological framing.

## From [the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro03](/entities/the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro03.md) (2026-06-09)
- Configuration file for OpenClaw.
- Conversational defaults can be overridden.

## From [the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro07](/entities/the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro07.md) (2026-06-09)
- Is a Bootstrap Context File.
- Sets behavioral guardrails (Persona).

## From [the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro08](/entities/the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro08.md) (2026-06-09)
- Used for 'Surgical Response'.
- Eliminates narrative embellishments.
- Uses 'Unclear' or 'Unknown' labels when uncertain.

## From [gzmo-soul-merged-new-part2-micro06](/entities/gzmo-soul-merged-new-part2-micro06.md) (2026-06-10)
- Defines the core identity of the OpenClaw agent.
- Located in the Workspace/Core Identity (OpenClaw Workspace-Textdateien).
- Strictly forbidden from being directly edited by the agent; belongs to the User's domain.

## From [gzmo-soul-merged-new-part2-micro08](/entities/gzmo-soul-merged-new-part2-micro08.md) (2026-06-10)
- Kern-Direktive des Agenten GZMO
- Definiert Rollen, Verhaltensregeln und Grenzen

## From [openclaw-deep-research-part1-micro06](/entities/openclaw-deep-research-part1-micro06.md) (2026-06-10)
- A Markdown file defining the agent's identity, personality, and boundaries.

## From [openclaw-part1-micro02](/entities/openclaw-part1-micro02.md) (2026-06-10)
- The kernel of the agent's identity

## From [openclaw-part1-micro05](/entities/openclaw-part1-micro05.md) (2026-06-10)
- Acts as the immutable kernel of the agent operating system.
- Defines persona, behavioral rules, and ethical/operational guardrails.
- Enforces the directive 'Action > Performance'.

## From [the-agentic-operating-environment-a-synthesis-arc-micro01](/entities/the-agentic-operating-environment-a-synthesis-arc-micro01.md) (2026-06-10)
- The agent's personality file.
- Lives on a high-speed encrypted USB drive in the Portable Sovereign Node plan.
