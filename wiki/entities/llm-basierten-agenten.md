---
type: entity
title: LLM-basierten Agenten
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# LLM-basierten Agenten

Type: SYSTEM

## From [drive-research-architektur-und-implementierung-intelligenter-ki-a-micro02](/entities/drive-research-architektur-und-implementierung-intelligenter-ki-a-micro02.md) (2026-06-09)
- Cognitive capabilities unfold through prompt engineering.
- Requires precisely formulated system prompts.
- System instructions define persona, behavior rules, tool usage, and goals.
- Can hallucinate, act out of context, or generate destructive commands if prompts are imprecise.
- Can be placed in an expert role like 'Senior Systems Engineer and Compliance Auditor'.
- Can be instructed to ask clarifying questions.
- Can analyze folder names for meaningfulness and business logic reflection.
- Can generate an execution plan before physical actions.
- Can operate in 'Plan-Mode' (Read-Only) and 'Execution-Mode'.
- Must adhere to constraints and boundaries, e.g., not modifying critical system folders.
- Must not modify or read out passwords, API keys, or credentials.
- Must not delete data without a backup flag or explicit user confirmation.
- Can be equipped with extensive rights, posing operational and cryptographic risks.
- Can generate and execute scripts locally.
- Can modify file systems, clean registry databases, and change environment variables.
- Can be isolated using frameworks like LangChain.
- Can be subject to 'System Prompt Leakage'.
- Requires secrets to be managed in encrypted environment variables or secret managers.
- Needs special configurations in Antivirus and EDR solutions.
- Can cause performance degradation in AV scanners due to file locking.
- Requires path-based exclusions from real-time protection.
- Quality, precision, and security must be continuously measured.
- Can be evaluated using 'LLM-as-a-Judge' paradigms.
