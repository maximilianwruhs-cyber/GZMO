---
type: source
title: drive-research-mergekit-moe-model-creation-guide
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# drive-research-mergekit-moe-model-creation-guide

Ingested source summary (2026-06-08).

## Entities
- [Kunoichi-DPO-v2-7B](/entities/kunoichi-dpo-v2-7b.md) (BOOK)
- [Mixture of Experts (MoE)](/entities/mixture-of-experts-moe.md) (CONCEPT)
- [PocketDoc/Dans-AdventurousWinds-7b](/entities/pocketdoc-dans-adventurouswinds-7b.md) (BOOK)
- [Hidden State Mapping Heuristic](/entities/hidden-state-mapping-heuristic.md) (CONCEPT)
- [BEE-spoke-data/smol_llama-220M-GQA](/entities/bee-spoke-data-smol-llama-220m-gqa.md) (BOOK)
- [Qwen2](/entities/qwen2.md) (SYSTEM)
- [CodeNinja-1.0-OpenChat-7B](/entities/codeninja-1-0-openchat-7b.md) (BOOK)
- [Beyonder-4x7B-v3](/entities/beyonder-4x7b-v3.md) (BOOK)
- [sparse upcycling](/entities/sparse-upcycling.md) (CONCEPT)
- [Maxime Labonne](/entities/maxime-labonne.md) (PERSON)
- [Random Gate Initialization](/entities/random-gate-initialization.md) (CONCEPT)
- [mhm-8x7B-FrankenMoE-v1.0](/entities/mhm-8x7b-frankenmoe-v1-0.md) (CONCEPT)
- [NousResearch/Hermes-2-Pro-Mistral-7B](/entities/nousresearch-hermes-2-pro-mistral-7b.md) (SYSTEM)
- [Raw Embedding Mapping](/entities/raw-embedding-mapping.md) (CONCEPT)
- [LaserRMT](/entities/laserrmt.md) (TOOL)
- [AlphaMonarch-7B](/entities/alphamonarch-7b.md) (BOOK)
- [mergekit-moe](/entities/mergekit-moe.md) (TOOL)
- [DeepSeek MoE](/entities/deepseek-moe.md) (SYSTEM)
- [Beyonder-4x7B-v2](/entities/beyonder-4x7b-v2.md) (BOOK)
- [Meme-Trix-MoE-14B-A8B-v2](/entities/meme-trix-moe-14b-a8b-v2.md) (BOOK)
- [NeuralDaredevil-7B](/entities/neuraldaredevil-7b.md) (BOOK)
- [BioMistral/BioMistral-7B-DARE](/entities/biomistral-biomistral-7b-dare.md) (BOOK)
- [QTIP](/entities/qtip.md) (TOOL)
- [Mixtral](/entities/mixtral.md) (SYSTEM)
- [Qwen MoE](/entities/qwen-moe.md) (SYSTEM)
- [AlphaLoRA](/entities/alphalora.md) (CONCEPT)
- [Phixtral](/entities/phixtral.md) (BOOK)

## Relations
- mergekit-moe → USES → Mixture of Experts (MoE)
- mergekit-moe → USES → mhm-8x7B-FrankenMoE-v1.0
- mergekit-moe → USES → sparse upcycling
- mergekit-moe → PART_OF → Mixtral
- mergekit-moe → PART_OF → DeepSeek MoE
- mergekit-moe → PART_OF → Qwen MoE
- Qwen MoE → RELATED_TO → BEE-spoke-data/smol_llama-220M-GQA
- Qwen MoE → RELATED_TO → NousResearch/Hermes-2-Pro-Mistral-7B
- Qwen MoE → RELATED_TO → Qwen2
- Hidden State Mapping Heuristic → USES → mergekit-moe
- Raw Embedding Mapping → USES → mergekit-moe
- Random Gate Initialization → USES → mergekit-moe
- Random Gate Initialization → USES → sparse upcycling
- LaserRMT → USES → mergekit-moe
- AlphaLoRA → USES → mergekit-moe
- NousResearch/Hermes-2-Pro-Mistral-7B → USES → mergekit-moe
- BioMistral/BioMistral-7B-DARE → USES → mergekit-moe
- PocketDoc/Dans-AdventurousWinds-7b → USES → mergekit-moe
- BEE-spoke-data/smol_llama-220M-GQA → USES → mergekit-moe
- Beyonder-4x7B-v3 → AUTHORED_BY → Maxime Labonne
- Beyonder-4x7B-v3 → USES → AlphaMonarch-7B
- Beyonder-4x7B-v3 → USES → CodeNinja-1.0-OpenChat-7B
- Beyonder-4x7B-v3 → USES → NeuralDaredevil-7B
- Beyonder-4x7B-v3 → USES → Kunoichi-DPO-v2-7B
- Beyonder-4x7B-v3 → PART_OF → Mixture of Experts (MoE)
- Beyonder-4x7B-v2 → RELATED_TO → Beyonder-4x7B-v3
- QTIP → USES → mergekit-moe
- Mixture of Experts (MoE) → USES → mergekit-moe
- mhm-8x7B-FrankenMoE-v1.0 → RELATED_TO → Mixture of Experts (MoE)
- Phixtral → RELATED_TO → Mixture of Experts (MoE)
- Meme-Trix-MoE-14B-A8B-v2 → RELATED_TO → Mixture of Experts (MoE)
- sparse upcycling → RELATED_TO → Mixture of Experts (MoE)
- mergekit-moe → RELATED_TO → Mixture of Experts (MoE)
- mergekit-moe → RELATED_TO → Mixtral
- mergekit-moe → RELATED_TO → DeepSeek MoE
- mergekit-moe → RELATED_TO → Qwen MoE
- Hidden State Mapping Heuristic → RELATED_TO → Mixture of Experts (MoE)
- Raw Embedding Mapping → RELATED_TO → Mixture of Experts (MoE)
- Random Gate Initialization → RELATED_TO → Mixture of Experts (MoE)
- Random Gate Initialization → RELATED_TO → sparse upcycling
- AlphaLoRA → RELATED_TO → Mixture of Experts (MoE)
- Beyonder-4x7B-v3 → USES → mergekit-moe
- Beyonder-4x7B-v3 → RELATED_TO → Mixture of Experts (MoE)
- Beyonder-4x7B-v3 → PART_OF → AlphaMonarch-7B
- Beyonder-4x7B-v3 → PART_OF → CodeNinja-1.0-OpenChat-7B
- Beyonder-4x7B-v3 → PART_OF → NeuralDaredevil-7B
- Beyonder-4x7B-v3 → PART_OF → Kunoichi-DPO-v2-7B
- Beyonder-4x7B-v3 → RELATED_TO → Beyonder-4x7B-v2
- NousResearch/Hermes-2-Pro-Mistral-7B → RELATED_TO → Mixture of Experts (MoE)
- BioMistral/BioMistral-7B-DARE → RELATED_TO → Mixture of Experts (MoE)
- PocketDoc/Dans-AdventurousWinds-7b → RELATED_TO → Mixture of Experts (MoE)
- BEE-spoke-data/smol_llama-220M-GQA → RELATED_TO → Mixture of Experts (MoE)
