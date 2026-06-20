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
- [turboquant Python package](/entities/turboquant-python-package.md) (TOOL)
- [Boundary V Protection](/entities/boundary-v-protection.md) (CONCEPT)
- [llama.cpp](/entities/llama-cpp.md) (SYSTEM)
- [Grouped-Query Attention (GQA)](/entities/grouped-query-attention-gqa.md) (CONCEPT)
- [Medusa](/entities/medusa.md) (CONCEPT)
- [N-gram matching](/entities/n-gram-matching.md) (CONCEPT)
- [Triton kernels](/entities/triton-kernels.md) (TOOL)
- [Attention Sparsity](/entities/attention-sparsity.md) (CONCEPT)
- [ML-SpecQD](/entities/ml-specqd.md) (CONCEPT)
- [Prompt Lookup Decoding](/entities/prompt-lookup-decoding.md) (CONCEPT)
- [Assisted Decoding](/entities/assisted-decoding.md) (CONCEPT)
- [Mamba](/entities/mamba.md) (SYSTEM)
- [Multi-Token Prediction (MTP)](/entities/multi-token-prediction-mtp.md) (CONCEPT)
- [Suffix Decoding](/entities/suffix-decoding.md) (CONCEPT)
- [vLLM](/entities/vllm.md) (SYSTEM)
- [State Space Models (SSMs)](/entities/state-space-models-ssms.md) (SYSTEM)
- [Multi-Head Attention (MHA)](/entities/multi-head-attention-mha.md) (CONCEPT)
- [EAGLE](/entities/eagle.md) (CONCEPT)
- [Asymmetric Quantization](/entities/asymmetric-quantization.md) (CONCEPT)
- [Google Research](/entities/google-research.md) (ORGANIZATION)
- [PolarQuant](/entities/polarquant.md) (TOOL)
- [QJL residual error corrections](/entities/qjl-residual-error-corrections.md) (TOOL)
- [Hugging Face Transformers](/entities/hugging-face-transformers.md) (SYSTEM)
- [Metal GPU kernels](/entities/metal-gpu-kernels.md) (TOOL)

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
