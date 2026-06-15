---
type: entity
title: Chain-of-Thought
created: 2026-06-08
updated: 2026-06-09
sources: 3
tags: []
status: draft
gzmo_synthetic: true
---



# Chain-of-Thought

Type: CONCEPT

## From [[drive-research-hermes-session-storage-migration-analysis|drive-research-hermes-session-storage-migration-analysis]] (2026-06-08)
- Models that generate Chain-of-Thought can have reasoning process leaks.
- Reasoning flow should be hidden when `display.show_reasoning: false`.

## From [[gzmo-soul-merged-new-part1|gzmo-soul-merged-new-part1]] (2026-06-09)
- Reasoning method used within ReAct loops.
- Consumes significantly more tokens than necessary.

## From [[architectures-and-optimizations-for-speculative-de-micro03|architectures-and-optimizations-for-speculative-de-micro03]] (2026-06-09)
- Describes the logical path of decision-making.
- Is traceable in classic, text-based agent systems.
- Is hidden in RecursiveMAS.
