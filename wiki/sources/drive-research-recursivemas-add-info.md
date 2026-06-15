---
type: source
title: drive-research-recursivemas-add-info
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# drive-research-recursivemas-add-info

Ingested source summary (2026-06-08).

## Entities
- [[load-from-repo-py|load_from_repo.py]] (TOOL)
- [[run-py|run.py]] (TOOL)
- [[apache-2-0-license|Apache 2.0 License]] (CONCEPT)
- [[mixture-style|Mixture-Style]] (CONCEPT)
- [[llama3-2-1b|Llama3.2-1B]] (SYSTEM)
- [[recursivelink-adapters|RecursiveLink adapters]] (TOOL)
- [[gemma3-4b|Gemma3-4B]] (SYSTEM)
- [[deepseek-r1-distill-qwen-1-5b|DeepSeek-R1-Distill-Qwen-1.5B]] (SYSTEM)
- [[open-weights-models|Open-Weights models]] (CONCEPT)
- [[huggingface|HuggingFace]] (ORGANIZATION)
- [[rtx-50-series|RTX-50 Series]] (TOOL)
- [[rtx-4090|RTX-4090]] (TOOL)
- [[modeling-py|modeling.py]] (TOOL)
- [[qwen2-5-coder-3b|Qwen2.5-Coder-3B]] (SYSTEM)
- [[google-takeout|Google Takeout]] (TOOL)
- [[recursivemas|RecursiveMAS]] (PROJECT)
- [[biomistral-7b|BioMistral-7B]] (SYSTEM)
- [[innerlink-module|InnerLink-Module]] (TOOL)
- [[sequential-style|Sequential-Style]] (CONCEPT)
- [[backbone-model-sharing|Backbone Model Sharing]] (CONCEPT)
- [[hf-resolver-py|hf_resolver.py]] (TOOL)
- [[cuda|CUDA]] (CONCEPT)
- [[github|GitHub]] (ORGANIZATION)
- [[qwen3-1-7b|Qwen3-1.7B]] (SYSTEM)
- [[qwen3-5-4b|Qwen3.5-4B]] (SYSTEM)
- [[outerlink-module|OuterLink-Module]] (TOOL)

## Relations
- RecursiveMAS → USES → GitHub
- RecursiveMAS → USES → HuggingFace
- RecursiveMAS → USES → Apache 2.0 License
- RecursiveMAS → USES → Open-Weights models
- RecursiveMAS → USES → RecursiveLink adapters
- RecursiveMAS → PART_OF → Sequential-Style
- RecursiveMAS → PART_OF → Mixture-Style
- RecursiveMAS → USES → OuterLink-Module
- RecursiveMAS → USES → InnerLink-Module
- RecursiveMAS → USES → CUDA
- RecursiveMAS → USES → Backbone Model Sharing
- Sequential-Style → USES → Qwen3-1.7B
- Sequential-Style → USES → Llama3.2-1B
- Sequential-Style → USES → Gemma3-4B
- Sequential-Style → USES → Qwen3.5-4B
- Mixture-Style → USES → DeepSeek-R1-Distill-Qwen-1.5B
- Mixture-Style → USES → Qwen2.5-Coder-3B
- Mixture-Style → USES → BioMistral-7B
- run.py → PART_OF → RecursiveMAS
- modeling.py → DEFINES → RecursiveLink adapters
- load_from_repo.py → PART_OF → RecursiveMAS
- hf_resolver.py → PART_OF → RecursiveMAS
- RecursiveLink adapters → RELATED_TO → Open-Weights models
- Google Takeout → RELATED_TO → RecursiveMAS
