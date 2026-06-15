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
- [[kunoichi-dpo-v2-7b|Kunoichi-DPO-v2-7B]] (BOOK)
- [[mixture-of-experts-moe|Mixture of Experts (MoE)]] (CONCEPT)
- [[pocketdoc-dans-adventurouswinds-7b|PocketDoc/Dans-AdventurousWinds-7b]] (BOOK)
- [[hidden-state-mapping-heuristic|Hidden State Mapping Heuristic]] (CONCEPT)
- [[bee-spoke-data-smol-llama-220m-gqa|BEE-spoke-data/smol_llama-220M-GQA]] (BOOK)
- [[qwen2|Qwen2]] (SYSTEM)
- [[codeninja-1-0-openchat-7b|CodeNinja-1.0-OpenChat-7B]] (BOOK)
- [[beyonder-4x7b-v3|Beyonder-4x7B-v3]] (BOOK)
- [[sparse-upcycling|sparse upcycling]] (CONCEPT)
- [[maxime-labonne|Maxime Labonne]] (PERSON)
- [[random-gate-initialization|Random Gate Initialization]] (CONCEPT)
- [[mhm-8x7b-frankenmoe-v1-0|mhm-8x7B-FrankenMoE-v1.0]] (CONCEPT)
- [[nousresearch-hermes-2-pro-mistral-7b|NousResearch/Hermes-2-Pro-Mistral-7B]] (SYSTEM)
- [[raw-embedding-mapping|Raw Embedding Mapping]] (CONCEPT)
- [[laserrmt|LaserRMT]] (TOOL)
- [[alphamonarch-7b|AlphaMonarch-7B]] (BOOK)
- [[mergekit-moe|mergekit-moe]] (TOOL)
- [[deepseek-moe|DeepSeek MoE]] (SYSTEM)
- [[beyonder-4x7b-v2|Beyonder-4x7B-v2]] (BOOK)
- [[meme-trix-moe-14b-a8b-v2|Meme-Trix-MoE-14B-A8B-v2]] (BOOK)
- [[neuraldaredevil-7b|NeuralDaredevil-7B]] (BOOK)
- [[biomistral-biomistral-7b-dare|BioMistral/BioMistral-7B-DARE]] (BOOK)
- [[qtip|QTIP]] (TOOL)
- [[mixtral|Mixtral]] (SYSTEM)
- [[qwen-moe|Qwen MoE]] (SYSTEM)
- [[alphalora|AlphaLoRA]] (CONCEPT)
- [[phixtral|Phixtral]] (BOOK)

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
