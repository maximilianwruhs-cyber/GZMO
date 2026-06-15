---
type: entity
title: LLMLingua-2-small
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# LLMLingua-2-small

Type: TOOL

## From [[drive-research-llmlingua-cpu-leistung-und-leistungstests|drive-research-llmlingua-cpu-leistung-und-leistungstests]] (2026-06-08)
- Refers to the mBERT Base (Multilingual) model variant of LLMLingua-2.
- Extremely lightweight with minimal RAM requirements.
- Suitable for CPU operation and containerized environments.
- Second generation of LLMLingua.
- Treats prompt compression as a binary token classification problem.
- Uses data distillation from GPT-4.
- Employs a Transformer encoder on BERT level.
- Significantly faster than the original LLMLingua.
- Uses lightweight models suitable for CPU operation.
- Lowers token consumption drastically.
- Is a suitable candidate as a compression backend for Hermes.
- Represents a significant technological leap over the current 'BoL checkpoint-summary' approach.
- Is an ultimate lightweight alternative with a bidirectional encoder model.
- Can process 48,000 tokens in under three seconds.
- Eliminates token pruning redundant operations, lowers API costs by up to 80%, and significantly reduces end-to-end latency.
- A framework for prompt compression.
- Evaluated as a potential backend for the Hermes architecture.
- First generation used causal language model perplexity.
- Iterative, coarse-to-fine compression methodology.
- Has a budget controller for compression ratio or token limit.
- Reduces token consumption by 50% to 80%.
- Integrates seamlessly into LangChain and LlamaIndex.
- Acts as middleware in LangChain via LLMLinguaCompressor.
