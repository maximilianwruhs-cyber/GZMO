---
type: source
title: the-architecture-of-speculative-decoding-and-infer-part1-micro06
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# the-architecture-of-speculative-decoding-and-infer-part1-micro06

Ingested source summary (2026-06-09).

## Entities
- [[turboquant-python-package|turboquant Python package]] (TOOL)
- [[boundary-v-protection|Boundary V Protection]] (CONCEPT)
- [[llama-cpp|llama.cpp]] (SYSTEM)
- [[grouped-query-attention-gqa|Grouped-Query Attention (GQA)]] (CONCEPT)
- [[medusa|Medusa]] (CONCEPT)
- [[n-gram-matching|N-gram matching]] (CONCEPT)
- [[triton-kernels|Triton kernels]] (TOOL)
- [[attention-sparsity|Attention Sparsity]] (CONCEPT)
- [[ml-specqd|ML-SpecQD]] (CONCEPT)
- [[prompt-lookup-decoding|Prompt Lookup Decoding]] (CONCEPT)
- [[assisted-decoding|Assisted Decoding]] (CONCEPT)
- [[mamba|Mamba]] (SYSTEM)
- [[multi-token-prediction-mtp|Multi-Token Prediction (MTP)]] (CONCEPT)
- [[suffix-decoding|Suffix Decoding]] (CONCEPT)
- [[vllm|vLLM]] (SYSTEM)
- [[state-space-models-ssms|State Space Models (SSMs)]] (SYSTEM)
- [[multi-head-attention-mha|Multi-Head Attention (MHA)]] (CONCEPT)
- [[eagle|EAGLE]] (CONCEPT)
- [[asymmetric-quantization|Asymmetric Quantization]] (CONCEPT)
- [[google-research|Google Research]] (ORGANIZATION)
- [[polarquant|PolarQuant]] (TOOL)
- [[qjl-residual-error-corrections|QJL residual error corrections]] (TOOL)
- [[hugging-face-transformers|Hugging Face Transformers]] (SYSTEM)
- [[metal-gpu-kernels|Metal GPU kernels]] (TOOL)

## Relations
- EAGLE → RELATED_TO → Medusa
- EAGLE → USES → turboquant Python package
- Medusa → USES → turboquant Python package
- Prompt Lookup Decoding → RELATED_TO → Suffix Decoding
- Prompt Lookup Decoding → USES → turboquant Python package
- Suffix Decoding → USES → turboquant Python package
- turboquant Python package → PART_OF → Hugging Face Transformers
- Assisted Decoding → USES → Hugging Face Transformers
- turboquant Python package → USES → vLLM
- Triton kernels → PART_OF → vLLM
- Asymmetric Quantization → USES → vLLM
- turboquant Python package → USES → llama.cpp
- Metal GPU kernels → PART_OF → llama.cpp
- Boundary V Protection → USES → llama.cpp
- Attention Sparsity → USES → llama.cpp
- turboquant Python package → USES → PolarQuant
- turboquant Python package → USES → QJL residual error corrections
- EAGLE → RELATED_TO → ML-SpecQD
- Medusa → RELATED_TO → ML-SpecQD
- Mamba → PART_OF → State Space Models (SSMs)
- turboquant Python package → RELATED_TO → State Space Models (SSMs)
- Multi-Head Attention (MHA) → RELATED_TO → turboquant Python package
