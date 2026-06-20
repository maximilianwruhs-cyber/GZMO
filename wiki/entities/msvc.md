---
type: entity
title: MSVC
created: 2026-06-08
updated: 2026-06-10
sources: 5
tags: []
status: draft
gzmo_synthetic: true
---





# MSVC

Type: TOOL

## From [drive-research-imagine-creating-sm120-according-to-our-progress](/entities/drive-research-imagine-creating-sm120-according-to-our-progress.md) (2026-06-08)
- If compiling in a Windows host environment, the NVCC compiler can trigger preprocessor failures (Error C1189) within the CUDA standard library headers (cccl) if MSVC's legacy preprocessor is engaged.
- Force compliance inside setup.py: if os.name == 'nt': extra_compile_args['cxx'] = ['/Zc:preprocessor', '/Zc:__cplusplus']

## From [drive-research-optimizing-cuda-performance-with-fp4-fp6-micro01](/entities/drive-research-optimizing-cuda-performance-with-fp4-fp6-micro01.md) (2026-06-09)
- A host compiler on Windows that has compatibility issues with CUDA Toolkit 13.1's automatic alignment of TMA descriptors.

## From [drive-research-optimizing-cuda-performance-with-fp4-fp6-micro02](/entities/drive-research-optimizing-cuda-performance-with-fp4-fp6-micro02.md) (2026-06-09)
- Windows MSVC environments require specific compiler option flags.
- NVCC can trigger preprocessor failures when compiled with MSVC.
- MSVC's legacy preprocessor can cause CUDA standard library header failures.

## From [optimizing-nvidia-blackwell-sm120-part1-micro07](/entities/optimizing-nvidia-blackwell-sm120-part1-micro07.md) (2026-06-10)
- A host compiler for Windows environments.
- Requires the /Zc:preprocessor and /Zc:__cplusplus flags to avoid C1189 errors when using the CUDA standard library.

## From [optimizing-nvidia-blackwell-sm120-part3-micro03](/entities/optimizing-nvidia-blackwell-sm120-part3-micro03.md) (2026-06-10)
- Toolchain used for compiling the engine on Windows.
