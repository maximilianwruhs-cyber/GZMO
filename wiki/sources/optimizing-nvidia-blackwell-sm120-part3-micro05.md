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
- [Remote Procedure Call (RPC)](/entities/remote-procedure-call-rpc.md) (CONCEPT)
- [ggml_cuda_graph_evaluate_and_capture](/entities/ggml-cuda-graph-evaluate-and-capture.md) (SYSTEM)
- [Qwen3.6-35B-A3B](/entities/qwen3-6-35b-a3b.md) (SYSTEM)
- [GGML_CUDA_DISABLE_GRAPHS=1](/entities/ggml-cuda-disable-graphs-1.md) (TOOL)
- [ggml_set_inplace](/entities/ggml-set-inplace.md) (TOOL)
- [CUDA Graph Execution Anomalies](/entities/cuda-graph-execution-anomalies.md) (CONCEPT)
- [ggml-cuda.cu](/entities/ggml-cuda-cu.md) (SYSTEM)
- [ggml-cuda Backend](/entities/ggml-cuda-backend.md) (SYSTEM)
- [ggml_top_k](/entities/ggml-top-k.md) (TOOL)
- [GGML_CUDA_USE_CUB](/entities/ggml-cuda-use-cub.md) (TOOL)
- [Gemma 4 31B](/entities/gemma-4-31b.md) (SYSTEM)
- [optimizing-nvidia-blackwell-sm120-part3](/entities/optimizing-nvidia-blackwell-sm120-part3.md) (PROJECT)
- [Mixture of Experts (MoE)](/entities/mixture-of-experts-moe.md) (CONCEPT)
- [Multi-Token Prediction (MTP)](/entities/multi-token-prediction-mtp.md) (CONCEPT)
- [Jinja parser](/entities/jinja-parser.md) (TOOL)
- [Sliding Window Attention (SWA)](/entities/sliding-window-attention-swa.md) (CONCEPT)
- [Non-P2P PCIe Topology Constraints](/entities/non-p2p-pcie-topology-constraints.md) (CONCEPT)
- [cuBLAS](/entities/cublas.md) (TOOL)
- [NVIDIA CUDA Collective Cooperatives Library (CCCL)](/entities/nvidia-cuda-collective-cooperatives-library-cccl.md) (TOOL)

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
