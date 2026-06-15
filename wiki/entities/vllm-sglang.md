---
type: entity
title: vLLM / SGLang
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# vLLM / SGLang

Type: TOOL

## From [[drive-research-imagine-creating-sm120-according-to-our-progress|drive-research-imagine-creating-sm120-according-to-our-progress]] (2026-06-08)
- Five core configuration files (trtllm_nvfp4_moe.py, flashinfer_trtllm_moe.py, flashinfer_cutedsl_moe.py, trtllm_fp8_moe.py, and cutlass_moe.py) lack capability family checks for SM120.
- They only check for is_device_capability_family(100), causing catastrophic fallback or launch failures on consumer GPUs.
- These files must be patched to append family 120.
- vLLM / SGLang: Five core configuration files lack capability family checks for SM120.
- Execute the serving instance with eager enforcement (capturing CUDA graphs during narrow-precision MoE runs on SM120 frequently triggers illegal memory access crashes under real traffic) and enforce global Marlin routing.
- docker run --gpus all -e VLLM_USE_FLASHINFER_MOE_FP4=0 -e VLLM_NVFP4_GEMM_BACKEND=marlin -e VLLM_TEST_FORCE_FP8_MARLIN=1 -e VLLM_MOE_FORCE_MARLIN=1 -p 8000:8000 vllm-node:latest python3 -m vllm.entrypoints.openai.api_server --model nvidia/Qwen3.5-397B-A17B-NVFP4 --tensor-parallel-size 4 --enforce-eager --kv-cache-dtype fp8_e4m3 --calculate-kv-scales
