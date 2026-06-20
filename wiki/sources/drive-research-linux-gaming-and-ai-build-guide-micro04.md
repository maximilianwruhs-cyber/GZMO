---
type: source
title: drive-research-linux-gaming-and-ai-build-guide-micro04
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# drive-research-linux-gaming-and-ai-build-guide-micro04

Ingested source summary (2026-06-09).

## Entities
- [AMD Radeon RX 9000 Series](/entities/amd-radeon-rx-9000-series.md) (TOOL)
- [NVIDIA Blackwell architecture](/entities/nvidia-blackwell-architecture.md) (CONCEPT)
- [vLLM](/entities/vllm.md) (TOOL)
- [Artificial Intelligence](/entities/artificial-intelligence.md) (CONCEPT)
- [Flash Attention 2](/entities/flash-attention-2.md) (TOOL)
- [DLSS 4.5](/entities/dlss-4-5.md) (CONCEPT)
- [ROCm 7.1](/entities/rocm-7-1.md) (TOOL)
- [Llama 70B model](/entities/llama-70b-model.md) (SYSTEM)
- [AMD RX 7900 XTX](/entities/amd-rx-7900-xtx.md) (TOOL)
- [CUDA cores](/entities/cuda-cores.md) (CONCEPT)
- [Generic Buffer Management API](/entities/generic-buffer-management-api.md) (CONCEPT)
- [FlashAttention 3](/entities/flashattention-3.md) (TOOL)
- [Tensor Cores](/entities/tensor-cores.md) (CONCEPT)
- [HIP assembly](/entities/hip-assembly.md) (CONCEPT)
- [PyTorch](/entities/pytorch.md) (TOOL)
- [NVIDIA proprietary Linux drivers](/entities/nvidia-proprietary-linux-drivers.md) (ORGANIZATION)
- [GDDR7 memory](/entities/gddr7-memory.md) (CONCEPT)
- [AMD MI355X](/entities/amd-mi355x.md) (TOOL)
- [Mesa 3D graphics library](/entities/mesa-3d-graphics-library.md) (TOOL)
- [Deep Learning Super Sampling](/entities/deep-learning-super-sampling.md) (CONCEPT)
- [PTX assembly](/entities/ptx-assembly.md) (CONCEPT)
- [MIOpen](/entities/miopen.md) (TOOL)
- [Premium Gaming](/entities/premium-gaming.md) (CONCEPT)
- [wlroots](/entities/wlroots.md) (SYSTEM)
- [Sway](/entities/sway.md) (SYSTEM)
- [Linux kernel](/entities/linux-kernel.md) (SYSTEM)
- [NVML](/entities/nvml.md) (CONCEPT)
- [NV-CONTROL API](/entities/nv-control-api.md) (CONCEPT)
- [GPU Dilemma](/entities/gpu-dilemma.md) (CONCEPT)
- [SGLang](/entities/sglang.md) (TOOL)
- [AMD RDNA 4/RDNA 5 platforms](/entities/amd-rdna-4-rdna-5-platforms.md) (CONCEPT)
- [AMD Mesa drivers](/entities/amd-mesa-drivers.md) (TOOL)
- [AI TOPS](/entities/ai-tops.md) (CONCEPT)
- [amdgpu driver](/entities/amdgpu-driver.md) (TOOL)
- [Warhammer 40K: Space Marine 2](/entities/warhammer-40k-space-marine-2.md) (BOOK)
- [Large Language Models](/entities/large-language-models.md) (CONCEPT)
- [explicit sync](/entities/explicit-sync.md) (CONCEPT)
- [CUDA Toolkit 13.0](/entities/cuda-toolkit-13-0.md) (TOOL)
- [Black Myth: Wukong](/entities/black-myth-wukong.md) (BOOK)
- [X11 protocol](/entities/x11-protocol.md) (SYSTEM)
- [GNOME](/entities/gnome.md) (SYSTEM)
- [Wayland](/entities/wayland.md) (SYSTEM)
- [Stable Diffusion XL](/entities/stable-diffusion-xl.md) (SYSTEM)
- [tile-based programming](/entities/tile-based-programming.md) (CONCEPT)
- [CUDA 13.0 ecosystem](/entities/cuda-13-0-ecosystem.md) (SYSTEM)
- [EGLStreams](/entities/eglstreams.md) (CONCEPT)
- [ComfyUI](/entities/comfyui.md) (TOOL)
- [RDNA 4 architecture](/entities/rdna-4-architecture.md) (CONCEPT)
- [Fluid Motion Frames 4](/entities/fluid-motion-frames-4.md) (CONCEPT)
- [Linux ecosystem](/entities/linux-ecosystem.md) (CONCEPT)
- [AMD MI300X](/entities/amd-mi300x.md) (TOOL)
- [KDE Plasma](/entities/kde-plasma.md) (SYSTEM)
- [TensorRT-LLM](/entities/tensorrt-llm.md) (TOOL)
- [Linux Workstation](/entities/linux-workstation.md) (SYSTEM)

## Relations
- AMD Radeon RX 9000 Series → PART_OF → RDNA 4 architecture
- AMD Radeon RX 9000 Series → USES → Premium Gaming
- AMD Radeon RX 9000 Series → USES → Linux Ecosystem
- NVIDIA proprietary Linux drivers → USES → Wayland
- Wayland → PART_OF → GNOME
- Wayland → PART_OF → KDE Plasma
- NVIDIA proprietary Linux drivers → USES → EGLStreams
- NVIDIA proprietary Linux drivers → USES → Generic Buffer Management API
- Wayland → USES → explicit sync
- NVIDIA proprietary Linux drivers → USES → explicit sync
- Sway → PART_OF → Wayland
- wlroots → PART_OF → Wayland
- NVIDIA proprietary Linux drivers → USES → NVML
- NVIDIA proprietary Linux drivers → USES → NV-CONTROL API
- CUDA Toolkit 13.0 → USES → tile-based programming
- FlashAttention 3 → USES → CUDA
- TensorRT-LLM → USES → CUDA
- NVIDIA proprietary Linux drivers → USES → CUDA
- ROCm 7.1 → USES → HIP assembly
- ROCm 7.1 → USES → PyTorch
- ROCm 7.1 → USES → vLLM
- ROCm 7.1 → USES → SGLang
- ROCm 7.1 → USES → Flash Attention 2
- Flash Attention 2 → USES → MIOpen
- PTX assembly → RELATED_TO → HIP assembly
- NVIDIA proprietary Linux drivers → USES → CUDA 13.0 ecosystem
- amdgpu driver → PART_OF → Linux kernel
- amdgpu driver → PART_OF → Mesa 3D graphics library
- AMD RX 7900 XTX → USES → Warhammer 40K: Space Marine 2
- NVIDIA proprietary Linux drivers → RELATED_TO → NVIDIA Blackwell architecture
- AMD Radeon RX 9000 Series → RELATED_TO → AMD RDNA 4/RDNA 5 platforms
- ROCm 7.1 → RELATED_TO → CUDA
- AMD Radeon RX 9000 Series → USES → ROCm 7.1
- AMD Radeon RX 9000 Series → USES → amdgpu driver
- AMD Radeon RX 9000 Series → USES → Mesa 3D graphics library
- AMD Radeon RX 9000 Series → USES → Fluid Motion Frames 4
- NVIDIA proprietary Linux drivers → RELATED_TO → AMD MI300X
- NVIDIA proprietary Linux drivers → RELATED_TO → AMD MI355X
- Linux Workstation → USES → Artificial Intelligence
- Linux Workstation → USES → Premium Gaming
- Linux Workstation → PART_OF → Linux ecosystem
- Stable Diffusion XL → USES → ComfyUI
- NVIDIA proprietary Linux drivers → USES → Linux ecosystem
- NVIDIA proprietary Linux drivers → USES → Linux drivers
- AMD Radeon RX 9000 Series → USES → Linux drivers
- NVIDIA proprietary Linux drivers → USES → CUDA Toolkit 13.0
- NVIDIA proprietary Linux drivers → USES → FlashAttention 3
- NVIDIA proprietary Linux drivers → USES → TensorRT-LLM
- AMD Radeon RX 9000 Series → USES → HIP assembly
- AMD Radeon RX 9000 Series → USES → Flash Attention 2
- AMD Radeon RX 9000 Series → USES → MIOpen
- NVIDIA proprietary Linux drivers → USES → PTX assembly
