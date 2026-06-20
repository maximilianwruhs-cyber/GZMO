---
type: source
title: drive-research-cuda-graph-capture-failure-workarounds-micro03
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# drive-research-cuda-graph-capture-failure-workarounds-micro03

Ingested source summary (2026-06-09).

## Entities
- [GEMV matrix-vector multiplication](/entities/gemv-matrix-vector-multiplication.md) (CONCEPT)
- [perf: PP graph reuse disable (#20463) causes 16% decode regression on V100 · Issue #20605 · ggml-org/llama.cpp](/entities/perf-pp-graph-reuse-disable-20463-causes-16-decode-regression-on-v100-issue-20605-ggml-org-llama-cpp.md) (CONCEPT)
- [instantiation failures](/entities/instantiation-failures.md) (CONCEPT)
- [NVIDIA CUDA Collective Cooperatives Library](/entities/nvidia-cuda-collective-cooperatives-library.md) (TOOL)
- [concurrent execution streams](/entities/concurrent-execution-streams.md) (CONCEPT)
- [non-CUDA workloads](/entities/non-cuda-workloads.md) (CONCEPT)
- [Tesla V100](/entities/tesla-v100.md) (SYSTEM)
- [token decoding](/entities/token-decoding.md) (CONCEPT)
- [virtual memory](/entities/virtual-memory.md) (CONCEPT)
- [Eval bug: RPC server leaks CUDA graphs during inference, leading to OOM #20315](/entities/eval-bug-rpc-server-leaks-cuda-graphs-during-inference-leading-to-oom-20315.md) (BOOK)
- [Eval bug: cudaGraphInstantiate OOM during prompt-cache checkpoint creation under sustained load (Q4_K_M MoE + --n-cpu-moe) · Issue #22638 · ggml-org/llama.cpp](/entities/eval-bug-cudagraphinstantiate-oom-during-prompt-cache-checkpoint-creation-under-sustained-load-q4-k-m-moe-n-cpu-moe-issue-22638-ggml-org-llama-cpp.md) (BOOK)
- [Eval bug: Tensor split failure (--split-mode row) · Issue #21404 · ggml-org/llama.cpp](/entities/eval-bug-tensor-split-failure-split-mode-row-issue-21404-ggml-org-llama-cpp.md) (BOOK)
- [n-cpu-moe; --cache-ram 0 does not disable checkpoints) — resubmit of #22638 · Issue #23181 · ggml-org/llama.cpp](/entities/n-cpu-moe-cache-ram-0-does-not-disable-checkpoints-resubmit-of-22638-issue-23181-ggml-org-llama-cpp.md) (BOOK)
- [Misc. bug: CUDA ggml_top_k() implementation crashes for large tensor shapes · Issue #21162 · ggml-org/llama.cpp](/entities/misc-bug-cuda-ggml-top-k-implementation-crashes-for-large-tensor-shapes-issue-21162-ggml-org-llama-cpp.md) (BOOK)
- [cache leaks](/entities/cache-leaks.md) (CONCEPT)
- [systemd](/entities/systemd.md) (TOOL)
- [host CPU](/entities/host-cpu.md) (SYSTEM)
- [GGML_CUDA_GRAPH_OPT](/entities/ggml-cuda-graph-opt.md) (TOOL)
- [gated activations](/entities/gated-activations.md) (CONCEPT)
- [GGML_CUDA_USE_CUB](/entities/ggml-cuda-use-cub.md) (TOOL)
- [tensor-split parameters](/entities/tensor-split-parameters.md) (TOOL)
- [Peer-to-Peer memory mappings](/entities/peer-to-peer-memory-mappings.md) (CONCEPT)
- [high batch sizes](/entities/high-batch-sizes.md) (CONCEPT)
- [attention projection computations](/entities/attention-projection-computations.md) (CONCEPT)
- [Eval bug: Memory leak on RPC CUDA backend · Issue #21265 · ggml-org/llama.cpp](/entities/eval-bug-memory-leak-on-rpc-cuda-backend-issue-21265-ggml-org-llama-cpp.md) (BOOK)
- [illegal memory access errors](/entities/illegal-memory-access-errors.md) (CONCEPT)
- [GGML_CUDA_DISABLE_GRAPHS](/entities/ggml-cuda-disable-graphs.md) (TOOL)
- [pipeline-parallel (layer-split) execution modes](/entities/pipeline-parallel-layer-split-execution-modes.md) (CONCEPT)
- [NVLink](/entities/nvlink.md) (SYSTEM)
- [host-staged transfers](/entities/host-staged-transfers.md) (CONCEPT)
- [context limits](/entities/context-limits.md) (CONCEPT)
- [slot context checkpoints](/entities/slot-context-checkpoints.md) (CONCEPT)
- [hardware memory boundaries](/entities/hardware-memory-boundaries.md) (CONCEPT)
- [thermal throttling](/entities/thermal-throttling.md) (CONCEPT)
- [asymmetric graphics cards](/entities/asymmetric-graphics-cards.md) (SYSTEM)
- [Eval bug: CUDA crash on tensor copy · Issue #20146 · ggml-org/llama.cpp](/entities/eval-bug-cuda-crash-on-tensor-copy-issue-20146-ggml-org-llama-cpp.md) (BOOK)
- [motherboard slots](/entities/motherboard-slots.md) (SYSTEM)
- [Eval bug: Segfault with Gemma 4 31B at - ggml-org/llama.cpp - GitHub](/entities/eval-bug-segfault-with-gemma-4-31b-at-ggml-org-llama-cpp-github.md) (BOOK)
- [Eval bug: Gemma 4 generates](/entities/eval-bug-gemma-4-generates.md) (BOOK)
- [Eval bug: CUDA ERROR crash when using MTP ngram-mod · Issue #23154 · ggml-org/llama.cpp](/entities/eval-bug-cuda-error-crash-when-using-mtp-ngram-mod-issue-23154-ggml-org-llama-cpp.md) (BOOK)
- [server deployments](/entities/server-deployments.md) (CONCEPT)
- [Optimizing Token Generation in llama.cpp's CUDA Backend #17621](/entities/optimizing-token-generation-in-llama-cpp-s-cuda-backend-17621.md) (BOOK)
- [large context lengths](/entities/large-context-lengths.md) (CONCEPT)
- [peer-to-peer transport latency](/entities/peer-to-peer-transport-latency.md) (CONCEPT)
- [client-side retry logic](/entities/client-side-retry-logic.md) (TOOL)
- [device memory states](/entities/device-memory-states.md) (CONCEPT)
- [Misc. bug: Multi-GPU layer split produces garbage output at context >2048 on non-P2P (CNS) PCIe topology (dual RTX 3090, B550) · Issue #20052 · ggml-org/llama.cpp](/entities/misc-bug-multi-gpu-layer-split-produces-garbage-output-at-context-2048-on-non-p2p-cns-pcie-topology-dual-rtx-3090-b550-issue-20052-ggml-org-llama-cpp.md) (BOOK)
- [Misc. bug: dual-GPU inference produces gibberish on asymmetric PCIe topology, while single-GPU works and older build b7728 works correctly · Issue #21887 · ggml-org/llama.cpp](/entities/misc-bug-dual-gpu-inference-produces-gibberish-on-asymmetric-pcie-topology-while-single-gpu-works-and-older-build-b7728-works-correctly-issue-21887-ggml-org-llama-cpp.md) (BOOK)
- [Remote Procedure Call](/entities/remote-procedure-call.md) (CONCEPT)
- [Mixture of Experts](/entities/mixture-of-experts.md) (CONCEPT)
- [VRAM ratios](/entities/vram-ratios.md) (CONCEPT)
- [vision-language models](/entities/vision-language-models.md) (CONCEPT)
- [Misc. bug: CUDA memory leak with MTMD requests loop with Gemma3 · Issue #19639 · ggml-org/llama.cpp](/entities/misc-bug-cuda-memory-leak-with-mtmd-requests-loop-with-gemma3-issue-19639-ggml-org-llama-cpp.md) (BOOK)
- [low-bandwidth riser lanes](/entities/low-bandwidth-riser-lanes.md) (SYSTEM)
- [Eval bug: Cuda error on split mode row after tensor parallelism changes #21773 - GitHub](/entities/eval-bug-cuda-error-on-split-mode-row-after-tensor-parallelism-changes-21773-github.md) (BOOK)
- [system supervisor watchdog](/entities/system-supervisor-watchdog.md) (TOOL)
- [advanced neural architectures](/entities/advanced-neural-architectures.md) (CONCEPT)
- [Eval bug: LLAMA_SET_ROWS=1 gibberish output with Dual GPU offload #14795 - GitHub](/entities/eval-bug-llama-set-rows-1-gibberish-output-with-dual-gpu-offload-14795-github.md) (BOOK)
- [Eval bug: core dumped at ggml_cuda_graph_evaluate_and_capture · Issue #20027 · ggml-org/llama.cpp](/entities/eval-bug-core-dumped-at-ggml-cuda-graph-evaluate-and-capture-issue-20027-ggml-org-llama-cpp.md) (BOOK)
- [CCCL](/entities/cccl.md) (TOOL)
- [PR #20463](/entities/pr-20463.md) (PROJECT)
- [memory manager](/entities/memory-manager.md) (SYSTEM)
- [Misc. bug: ggml-cuda\ggml-cuda.cu:98: CUDA error #21289 - GitHub](/entities/misc-bug-ggml-cuda-ggml-cuda-cu-98-cuda-error-21289-github.md) (BOOK)
- [Volta-class accelerators](/entities/volta-class-accelerators.md) (SYSTEM)
- [parallel algorithms](/entities/parallel-algorithms.md) (CONCEPT)
- [single-GPU execution](/entities/single-gpu-execution.md) (CONCEPT)
- [mathematical reduction operations](/entities/mathematical-reduction-operations.md) (CONCEPT)
- [CUDA illegal memory access with Qwen3-Next on multi-GPU using -ot (regression) · Issue #19816 · ggml-org/llama.cpp](/entities/cuda-illegal-memory-access-with-qwen3-next-on-multi-gpu-using-ot-regression-issue-19816-ggml-org-llama-cpp.md) (BOOK)
- [MoE experts](/entities/moe-experts.md) (CONCEPT)
- [P2P routing](/entities/p2p-routing.md) (CONCEPT)

## Relations
- GGML_CUDA_GRAPH_OPT → USES → attention projection computations
- PR #20463 → PART_OF → perf: PP graph reuse disable (#20463) causes 16% decode regression on V100 · Issue #20605 · ggml-org/llama.cpp
- PR #20463 → PART_OF → pipeline-parallel (layer-split) execution modes
- Volta-class accelerators → RELATED_TO → perf: PP graph reuse disable (#20463) causes 16% decode regression on V100 · Issue #20605 · ggml-org/llama.cpp
- host CPU → RELATED_TO → perf: PP graph reuse disable (#20463) causes 16% decode regression on V100 · Issue #20605 · ggml-org/llama.cpp
- host CPU → USES → token decoding
- GGML_CUDA_DISABLE_GRAPHS → PART_OF → server deployments
- GGML_CUDA_DISABLE_GRAPHS → PART_OF → vision-language models
- Remote Procedure Call → PART_OF → server deployments
- NVIDIA CUDA Collective Cooperatives Library → RELATED_TO → advanced neural architectures
- Peer-to-Peer memory mappings → PART_OF → motherboard slots
- systemd → USES → Mixture of Experts
- systemd → RELATED_TO → system supervisor watchdog
- Mixture of Experts → RELATED_TO → slot context checkpoints
- NVIDIA CUDA Collective Cooperatives Library → RELATED_TO → CCCL
- NVIDIA CUDA Collective Cooperatives Library → USES → parallel algorithms
- CCCL → USES → GGML_CUDA_USE_CUB
- CCCL → USES → parallel algorithms
- GGML_CUDA_USE_CUB → USES → parallel algorithms
- Eval bug: cudaGraphInstantiate OOM during prompt-cache checkpoint creation under sustained load (Q4_K_M MoE + --n-cpu-moe) · Issue #22638 · ggml-org/llama.cpp → USES → NVIDIA CUDA Collective Cooperatives Library
- perf: PP graph reuse disable (#20463) causes 16% decode regression on V100 · Issue #20605 · ggml-org/llama.cpp → RELATED_TO → pipeline-parallel (layer-split) execution modes
- perf: PP graph reuse disable (#20463) causes 16% decode regression on V100 · Issue #20605 · ggml-org/llama.cpp → PART_OF → PR #20463
- pipeline-parallel (layer-split) execution modes → RELATED_TO → perf: PP graph reuse disable (#20463) causes 16% decode regression on V100 · Issue #20605 · ggml-org/llama.cpp
- Optimizing Token Generation in llama.cpp's CUDA Backend #17621 → USES → GEMV matrix-vector multiplication
- Optimizing Token Generation in llama.cpp's CUDA Backend #17621 → USES → gated activations
- Optimizing Token Generation in llama.cpp's CUDA Backend #17621 → USES → attention projection computations
- attention projection computations → USES → concurrent execution streams
- memory manager → RELATED_TO → concurrent execution streams
- pipeline-parallel (layer-split) execution modes → PART_OF → PR #20463
- host CPU → PART_OF → Tesla V100
- token decoding → USES → host CPU
- thermal throttling → RELATED_TO → host CPU
- perf: PP graph reuse disable (#20463) causes 16% decode regression on V100 · Issue #20605 · ggml-org/llama.cpp → RELATED_TO → peer-to-peer transport latency
- perf: PP graph reuse disable (#20463) causes 16% decode regression on V100 · Issue #20605 · ggml-org/llama.cpp → RELATED_TO → host-staged transfers
- host-staged transfers → RELATED_TO → P2P routing
- advanced neural architectures → USES → NVIDIA CUDA Collective Cooperatives Library
- server deployments → USES → Remote Procedure Call
- vision-language models → PART_OF → server deployments
- GGML_CUDA_DISABLE_GRAPHS → RELATED_TO → cache leaks
- GGML_CUDA_DISABLE_GRAPHS → RELATED_TO → instantiation failures
- asymmetric graphics cards → RELATED_TO → tensor-split parameters
- tensor-split parameters → RELATED_TO → VRAM ratios
- motherboard slots → RELATED_TO → P2P routing
- low-bandwidth riser lanes → RELATED_TO → P2P routing
- context limits → RELATED_TO → P2P routing
- single-GPU execution → RELATED_TO → host-staged transfers
- system supervisor watchdog → RELATED_TO → client-side retry logic
- slot context checkpoints → RELATED_TO → virtual memory
- slot context checkpoints → RELATED_TO → device memory states
- illegal memory access errors → RELATED_TO → mathematical reduction operations
- illegal memory access errors → RELATED_TO → NVIDIA CUDA Collective Cooperatives Library
- parallel algorithms → RELATED_TO → hardware memory boundaries
- Optimizing Token Generation in llama.cpp's CUDA Backend #17621 → PART_OF → Eval bug: cudaGraphInstantiate OOM during prompt-cache checkpoint creation under sustained load (Q4_K_M MoE + --n-cpu-moe) · Issue #22638 · ggml-org/llama.cpp
- Eval bug: RPC server leaks CUDA graphs during inference, leading to OOM #20315 → RELATED_TO → NVIDIA CUDA Collective Cooperatives Library
- Eval bug: cudaGraphInstantiate OOM during prompt-cache checkpoint creation under sustained load (Q4_K_M MoE + --n-cpu-moe) · Issue #22638 · ggml-org/llama.cpp → PART_OF → Mixture of Experts
- CUDA illegal memory access with Qwen3-Next on multi-GPU using -ot (regression) · Issue #19816 · ggml-org/llama.cpp → RELATED_TO → NVIDIA CUDA Collective Cooperatives Library
- Eval bug: Tensor split failure (--split-mode row) · Issue #21404 · ggml-org/llama.cpp → RELATED_TO → tensor split
- Eval bug: Cuda error on split mode row after tensor parallelism changes #21773 - GitHub → RELATED_TO → NVIDIA CUDA Collective Cooperatives Library
- Misc. bug: Multi-GPU layer split produces garbage output at context >2048 on non-P2P (CNS) PCIe topology (dual RTX 3090, B550) · Issue #20052 · ggml-org/llama.cpp → RELATED_TO → P2P routing
- Misc. bug: CUDA memory leak with MTMD requests loop with Gemma3 · Issue #19639 · ggml-org/llama.cpp → RELATED_TO → NVIDIA CUDA Collective Cooperatives Library
- Eval bug: CUDA ERROR crash when using MTP ngram-mod · Issue #23154 · ggml-org/llama.cpp → RELATED_TO → NVIDIA CUDA Collective Cooperatives Library
- Eval bug: Memory leak on RPC CUDA backend · Issue #21265 · ggml-org/llama.cpp → RELATED_TO → NVIDIA CUDA Collective Cooperatives Library
- n-cpu-moe; --cache-ram 0 does not disable checkpoints) — resubmit of #22638 · Issue #23181 · ggml-org/llama.cpp → PART_OF → Mixture of Experts
- Misc. bug: ggml-cuda\ggml-cuda.cu:98: CUDA error #21289 - GitHub → RELATED_TO → NVIDIA CUDA Collective Cooperatives Library
- perf: PP graph reuse disable (#20463) causes 16% decode regression on V100 · Issue #20605 · ggml-org/llama.cpp → RELATED_TO → token decoding
- Eval bug: core dumped at ggml_cuda_graph_evaluate_and_capture · Issue #20027 · ggml-org/llama.cpp → RELATED_TO → NVIDIA CUDA Collective Cooperatives Library
- Eval bug: CUDA crash on tensor copy · Issue #20146 · ggml-org/llama.cpp → RELATED_TO → NVIDIA CUDA Collective Cooperatives Library
- Misc. bug: CUDA ggml_top_k() implementation crashes for large tensor shapes · Issue #21162 · ggml-org/llama.cpp → RELATED_TO → NVIDIA CUDA Collective Cooperatives Library
