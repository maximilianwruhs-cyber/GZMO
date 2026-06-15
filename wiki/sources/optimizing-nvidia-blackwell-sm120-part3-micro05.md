---
type: source
title: optimizing-nvidia-blackwell-sm120-part3-micro05
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# optimizing-nvidia-blackwell-sm120-part3-micro05

Ingested source summary (2026-06-09).

## Entities
- [[remote-procedure-call-rpc|Remote Procedure Call (RPC)]] (CONCEPT)
- [[ggml-cuda-graph-evaluate-and-capture|ggml_cuda_graph_evaluate_and_capture]] (SYSTEM)
- [[qwen3-6-35b-a3b|Qwen3.6-35B-A3B]] (SYSTEM)
- [[ggml-cuda-disable-graphs-1|GGML_CUDA_DISABLE_GRAPHS=1]] (TOOL)
- [[ggml-set-inplace|ggml_set_inplace]] (TOOL)
- [[cuda-graph-execution-anomalies|CUDA Graph Execution Anomalies]] (CONCEPT)
- [[ggml-cuda-cu|ggml-cuda.cu]] (SYSTEM)
- [[ggml-cuda-backend|ggml-cuda Backend]] (SYSTEM)
- [[ggml-top-k|ggml_top_k]] (TOOL)
- [[ggml-cuda-use-cub|GGML_CUDA_USE_CUB]] (TOOL)
- [[gemma-4-31b|Gemma 4 31B]] (SYSTEM)
- [[optimizing-nvidia-blackwell-sm120-part3|optimizing-nvidia-blackwell-sm120-part3]] (PROJECT)
- [[mixture-of-experts-moe|Mixture of Experts (MoE)]] (CONCEPT)
- [[multi-token-prediction-mtp|Multi-Token Prediction (MTP)]] (CONCEPT)
- [[jinja-parser|Jinja parser]] (TOOL)
- [[sliding-window-attention-swa|Sliding Window Attention (SWA)]] (CONCEPT)
- [[non-p2p-pcie-topology-constraints|Non-P2P PCIe Topology Constraints]] (CONCEPT)
- [[cublas|cuBLAS]] (TOOL)
- [[nvidia-cuda-collective-cooperatives-library-cccl|NVIDIA CUDA Collective Cooperatives Library (CCCL)]] (TOOL)

## Relations
- CUDA Graph Execution Anomalies → PART_OF → ggml-cuda Backend
- ggml-cuda Backend → USES → GGML_CUDA_DISABLE_GRAPHS=1
- GGML_CUDA_DISABLE_GRAPHS=1 → RELATED_TO → Remote Procedure Call (RPC)
- Gemma 4 31B → USES → cuBLAS
- Gemma 4 31B → USES → Sliding Window Attention (SWA)
- Multi-Token Prediction (MTP) → PART_OF → ggml-cuda Backend
- Mixture of Experts (MoE) → PART_OF → ggml-cuda Backend
- Qwen3.6-35B-A3B → RELATED_TO → Mixture of Experts (MoE)
- ggml_cuda_graph_evaluate_and_capture → RELATED_TO → Mixture of Experts (MoE)
- ggml_set_inplace → RELATED_TO → ggml_cuda_graph_evaluate_and_capture
- ggml-cuda.cu → PART_OF → ggml-cuda Backend
- ggml_top_k → USES → NVIDIA CUDA Collective Cooperatives Library (CCCL)
- ggml_top_k → USES → GGML_CUDA_USE_CUB
- Non-P2P PCIe Topology Constraints → RELATED_TO → ggml-cuda Backend
- Sliding Window Attention (SWA) → USES → Gemma 4 31B
- Jinja parser → USES → Gemma 4 31B
- optimizing-nvidia-blackwell-sm120-part3 → RELATED_TO → CUDA Graph Execution Anomalies
