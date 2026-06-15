---
type: entity
title: Llama-3-8B
created: 2026-06-08
updated: 2026-06-10
sources: 10
tags: []
status: draft
gzmo_synthetic: true
---










# Llama-3-8B

Type: MODEL

## From [[architectural-blueprints-for-sovereign-frankenmoe-part1|architectural-blueprints-for-sovereign-frankenmoe-part1]] (2026-06-08)
- An example of a base model for homogeneous merging.
- Can be merged with fine-tuned variations like python-coding and medical QA models.
- Used as 'base_model' in a sample mergekit configuration.
- An example of a common base model for homogeneous merging.
- Models fine-tuned from it reside within the same loss basin.
- Used to illustrate homogeneous merging.
- It is a type of checkpoint used in ensembling for FrankenMoE models.
- Specialized Llama checkpoints were used in Meme-Trix-MoE-14B-A8B-v2 and mhm-8x7B-FrankenMoE-v1.0.
- It is a pre-trained dense model.

## From [[drive-research-hidden-mode-technical-analysis-and-configuration|drive-research-hidden-mode-technical-analysis-and-configuration]] (2026-06-08)
- A common base model for homogeneous merging.
- Variations fine-tuned from this model can be merged.

## From [[drive-research-research-project-initiation-guide|drive-research-research-project-initiation-guide]] (2026-06-08)
- A language model used in local, reproducible experiments.
- Observed trade-offs when using generic prompt templates.
- Showed reduced information extraction pass rate and RAG compliance with generic templates.

## From [[obolus-vs-codium-extension-konzept-research-part1-micro05|obolus-vs-codium-extension-konzept-research-part1-micro05]] (2026-06-09)
- A model compared in the Leaderboard.

## From [[prompt-agent-engineering-part4-micro01|prompt-agent-engineering-part4-micro01]] (2026-06-09)
- An example of an Emergency-LLM used in Island-Mode.

## From [[drive-research-research-process-steps-micro03|drive-research-research-process-steps-micro03]] (2026-06-10)
- Used to model VRAM consumption with a 32K context window.
- Can be quantized to Q4_K_M weights.

## From [[optimizing-nvidia-blackwell-sm120-part2-micro04|optimizing-nvidia-blackwell-sm120-part2-micro04]] (2026-06-10)
- Model used for performance benchmarking.

## From [[optimizing-nvidia-blackwell-sm120-part2-micro07|optimizing-nvidia-blackwell-sm120-part2-micro07]] (2026-06-10)
- Used in local, reproducible experiments
- RAG compliance reduced from 93.3% to 80% when using generic templates

## From [[optimizing-nvidia-blackwell-sm120-part3-micro02|optimizing-nvidia-blackwell-sm120-part3-micro02]] (2026-06-10)
- Can be run with Q4_K_M weights
- Can be modeled with a 32K context window

## From [[prompt-agent-engineering-part4-micro02|prompt-agent-engineering-part4-micro02]] (2026-06-10)
- Used as a local LLM for the Emergency-LLM or baseline instinct engine
