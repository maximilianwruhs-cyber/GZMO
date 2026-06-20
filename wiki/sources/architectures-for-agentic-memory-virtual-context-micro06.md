---
type: source
title: architectures-for-agentic-memory-virtual-context-micro06
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# architectures-for-agentic-memory-virtual-context-micro06

Ingested source summary (2026-06-09).

## Entities
- [Episodic memory](/entities/episodic-memory.md) (CONCEPT)
- [Core Memory](/entities/core-memory.md) (CONCEPT)
- [Endel Tulving](/entities/endel-tulving.md) (PERSON)
- [Spreading Activation](/entities/spreading-activation.md) (CONCEPT)
- [OpenAI Responses API](/entities/openai-responses-api.md) (ORGANIZATION)
- [Recall Memory](/entities/recall-memory.md) (CONCEPT)
- [LanceDB](/entities/lancedb.md) (TOOL)
- [NEON-SOUL skill](/entities/neon-soul-skill.md) (TOOL)
- [fault-driven page pinning](/entities/fault-driven-page-pinning.md) (CONCEPT)
- [SEKG cognitive stack](/entities/sekg-cognitive-stack.md) (CONCEPT)
- [Zep Graphiti engine](/entities/zep-graphiti-engine.md) (SYSTEM)
- [bi-temporal tracking](/entities/bi-temporal-tracking.md) (CONCEPT)
- [Letta virtual context paradigm](/entities/letta-virtual-context-paradigm.md) (CONCEPT)
- [Mem0g](/entities/mem0g.md) (SYSTEM)
- [semantic memory](/entities/semantic-memory.md) (CONCEPT)
- [Knowledge Graphs](/entities/knowledge-graphs.md) (CONCEPT)
- [Letta V1 agent architecture](/entities/letta-v1-agent-architecture.md) (SYSTEM)
- [Claude 3.5 Sonnet](/entities/claude-3-5-sonnet.md) (SYSTEM)
- [Yield state](/entities/yield-state.md) (CONCEPT)
- [GPT-4o mini](/entities/gpt-4o-mini.md) (SYSTEM)
- [Pichay framework](/entities/pichay-framework.md) (SYSTEM)
- [context engineering](/entities/context-engineering.md) (CONCEPT)
- [SOUL.md](/entities/soul-md.md) (CONCEPT)
- [Large Language Models](/entities/large-language-models.md) (SYSTEM)
- [MemGPT](/entities/memgpt.md) (SYSTEM)
- [heartbeat](/entities/heartbeat.md) (CONCEPT)
- [thrashing](/entities/thrashing.md) (CONCEPT)
- [Soul, Episodic, and Knowledge Graph (SEKG) stack](/entities/soul-episodic-and-knowledge-graph-sekg-stack.md) (SYSTEM)
- [OpenClaw](/entities/openclaw.md) (SYSTEM)
- [Atkinson and Shiffrin multi-store model of memory encoding](/entities/atkinson-and-shiffrin-multi-store-model-of-memory-encoding.md) (CONCEPT)
- [Anthropic](/entities/anthropic.md) (ORGANIZATION)
- [Archival Memory](/entities/archival-memory.md) (CONCEPT)

## Relations
- Large Language Models → RELATED_TO → context engineering
- MemGPT → RELATED_TO → Large Language Models
- MemGPT → RELATED_TO → Letta virtual context paradigm
- SEKG stack → RELATED_TO → SEKG cognitive stack
- Letta virtual context paradigm → RELATED_TO → SEKG stack
- Letta virtual context paradigm → PART_OF → Core Memory
- Letta virtual context paradigm → PART_OF → Recall Memory
- Letta virtual context paradigm → PART_OF → Archival Memory
- Archival Memory → USES → LanceDB
- Letta virtual context paradigm → RELATED_TO → fault-driven page pinning
- Letta virtual context paradigm → RELATED_TO → Yield state
- Letta virtual context paradigm → RELATED_TO → heartbeat
- Letta V1 agent architecture → RELATED_TO → heartbeat
- Letta V1 agent architecture → USES → OpenAI Responses API
- GPT-4o mini → USES → Letta V1 agent architecture
- Claude 3.5 Sonnet → USES → Letta V1 agent architecture
- GPT-4o mini → RELATED_TO → Letta V1 agent architecture
- Letta virtual context paradigm → RELATED_TO → thrashing
- Pichay framework → RELATED_TO → fault-driven page pinning
- Endel Tulving → RELATED_TO → SEKG stack
- SEKG stack → PART_OF → NEON-SOUL skill
- SEKG stack → PART_OF → Episodic memory
- SEKG stack → PART_OF → semantic memory
- OpenClaw → RELATED_TO → NEON-SOUL skill
- OpenClaw → USES → SOUL.md
- NEON-SOUL skill → USES → SOUL.md
- Episodic memory → RELATED_TO → Atkinson and Shiffrin multi-store model of memory encoding
- Episodic memory → USES → Zep Graphiti engine
- Episodic memory → RELATED_TO → bi-temporal tracking
- semantic memory → RELATED_TO → Knowledge Graphs
- semantic memory → RELATED_TO → Spreading Activation
- Knowledge Graphs → RELATED_TO → Spreading Activation
- Claude 3.5 Sonnet → PART_OF → Anthropic
