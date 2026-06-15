---
type: entity
title: pi-context-injector
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# pi-context-injector

Type: PROJECT

## From [[drive-research-optimizing-pi-coding-agent-performance|drive-research-optimizing-pi-coding-agent-performance]] (2026-06-08)
- A project with GitHub repositories and issues.
- Has packages like 'coding-agent'.
- A GitHub repository related to 'pi'.
- Uses Zero-Catalog Mode with policy-based retrieval.
- Hides the default, bulky skill catalog from the first-turn prompt.
- Injects short suggestions instead of full files.
- Toggles skill visibility on-demand using nine hotkey-triggered session slots or explicit inline commands.
- Prevents automatic skill-discovery prompt injection.
- Keeps skills hidden from the LLM until explicitly invoked.
- Intercepts turn generation to inject context only during initial prompts or compaction events.
- Removes static contextual details from normal turn cycles.
- Keeps the intermediate context window compact.
- Operates on a philosophy of structural minimalism.
- Features a system prompt of under one thousand tokens.
- Exposes four core built-in tools: file reading (read), file writing (write), precise text-replacement patching (edit), and general shell execution (bash).
- Framework for implementing an 'AgenticOS' style architecture.
- Allows a primary coordinator agent to delegate specialized tasks to smaller, highly tuned local processes.
- Sub-agents are configured via markdown files containing YAML frontmatter.
- Automates the search process for optimizing code at the hardware level.
- Based on Andrej Karpathy's autoresearch paradigm.
- Turns any quantitative metric into an optimization target.
