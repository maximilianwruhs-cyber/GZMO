---
type: entity
title: GPT-5-mini
created: 2026-06-09
updated: 2026-06-09
sources: 3
tags: []
status: draft
gzmo_synthetic: true
---




# GPT-5-mini

Type: SYSTEM

## From [[ai-research-part3-micro01|ai-research-part3-micro01]] (2026-06-09)
- Used to optimize an agent on SWE-bench Verified.
- An agent optimized with this model achieves human-level performance on SWE-bench Lite.
- Used to evaluate an agent on SWE-bench Lite.

## From [[ai-research-part3-micro03|ai-research-part3-micro03]] (2026-06-09)
- HGM’s Best-belief SWE-Verified Agent + GPT-5 on SWE-Lite
- HGM generalizes across both dataset and model shifts, achieving human-level coding agent design performance on SWE-bench Lite with GPT-5 despite being optimized on SWE-bench Verified with GPT-5-mini
- human-designed agent built on GPT-5-mini
- top-scoring GPT-5-mini–based system
- systems built on stronger backbone models that can cost 5× more (e.g., Claude-3.7)
- no checked submission using GPT-5-mini appears on the SWE-Lite leaderboard
- adapting the leading system (with checked submissions) (SWE-agent + Claude 4 Sonnet) by replacing its backbone with GPT-5-mini, yielding SWE-agent + GPT-5-mini, as an additional baseline for comparison
- HGM’s Best-belief agent with GPT-5-mini
- the edge arises not from the GPT-5-mini backbone, but from the genuine design improvements introduced by HGM evolution

## From [[ai-research-part8-micro04|ai-research-part8-micro04]] (2026-06-09)
- A smaller backbone used by an HGM-driven agent.
- Successfully mutated into a framework that matched human-engineered coding agents.
