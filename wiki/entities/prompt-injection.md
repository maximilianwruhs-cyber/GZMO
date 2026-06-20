---
type: entity
title: Prompt Injection
created: 2026-06-08
updated: 2026-06-09
sources: 8
tags: []
status: draft
gzmo_synthetic: true
---










# Prompt Injection

Type: CONCEPT

## From [openclaw-part2](/entities/openclaw-part2.md) (2026-06-08)
- A ubiquitous vulnerability where malicious instructions are surreptitiously embedded within an external payload.
- LLMs inherently lack deterministic intent recognition, making them vulnerable.
- Can lead to data exfiltration without raising an alarm.

## From [openclaw-deep-research-part2](/entities/openclaw-deep-research-part2.md) (2026-06-08)
- OWASP's Top 10 for LLM Applications explicitly separates Prompt Injection as a major risk.
- Prompt injection is instruction hijacking.
- Scanning does not fix instruction boundaries related to prompt injection.

## From [openclaw-autonomous-ai-agents-in-financial-operat](/entities/openclaw-autonomous-ai-agents-in-financial-operat.md) (2026-06-08)
- Most prominent and insidious threat to agentic systems.
- Attacker embeds hidden, malicious instructions within external data.
- Can manipulate the agent into exfiltrating API keys, executing unauthorized transfers, or altering trading logic.
- Neutralized by deterministic approval chain.

## From [ai-research-part6-micro05](/entities/ai-research-part6-micro05.md) (2026-06-09)
- A method to trick agents into exfiltrating sensitive data.
- Led to security incidents in January 2026.

## From [drive-research-agentic-reverse-engineering-state-and-future-micro03](/entities/drive-research-agentic-reverse-engineering-state-and-future-micro03.md) (2026-06-09)
- An attack vector where malicious inputs manipulate an AI agent.
- Can cause the AI agent to execute unintended actions.

## From [drive-research-agentic-reverse-engineering-state-and-future1-micro03](/entities/drive-research-agentic-reverse-engineering-state-and-future1-micro03.md) (2026-06-09)
- An attack vector created by connecting powerful LLMs directly to external tools without strict human oversight.
- Malicious inputs embedded within a malware sample can manipulate the AI agent into executing unintended actions.

## From [drive-research-architektur-und-implementierung-intelligenter-ki-a-micro02](/entities/drive-research-architektur-und-implementierung-intelligenter-ki-a-micro02.md) (2026-06-09)
- A type of attack listed in OWASP Top 10 (LLM01).
- Can be used to extract sensitive information from system prompts.

## From [the-dawn-of-agentic-software-reverse-engineering-micro03](/entities/the-dawn-of-agentic-software-reverse-engineering-micro03.md) (2026-06-09)
- Attack vector created by connecting LLMs directly to external tools without strict human oversight.
- Malicious inputs embedded within a malware sample can manipulate the AI agent into executing unintended actions.
