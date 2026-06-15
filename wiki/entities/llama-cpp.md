---
type: entity
title: llama.cpp
created: 2026-06-08
updated: 2026-06-10
sources: 57
tags: []
status: draft
gzmo_synthetic: true
---





























































# llama.cpp

Type: TOOL

## From [[architecting-the-minimalist-linux-desktop-a-compa-part1|architecting-the-minimalist-linux-desktop-a-compa-part1]] (2026-06-08)
- A cross-platform binary that integrates highly specialized computational routines.
- Its Vulkan backend is heavily competitive with native CUDA, particularly regarding token generation metrics.
- Its Vulkan backend required approximately 5 Gigabytes less VRAM than the CUDA implementation in comparative testing.
- An inference engine.
- Can be routed through the Vulkan backend using the NVK user-space driver.

## From [[architectural-blueprints-for-sovereign-frankenmoe-part1|architectural-blueprints-for-sovereign-frankenmoe-part1]] (2026-06-08)
- Can be utilized for CPU-GPU hybrid execution in environments with limited VRAM.
- Uses the --cpu-moe (or -ot "exps=CPU") flag to offload components.
- Keeps N layers of experts in system RAM, starting from the highest-numbered layers, in default hybrid mode.
- Can be utilized for CPU-GPU hybrid execution.
- Supports offloading key-value (KV) cache, attention layers, and shared experts to the GPU.
- Uses the --cpu-moe flag (or -ot "exps=CPU") for offloading.
- It is used for local bare-metal execution of compiled weight space.
- It supports post-training quantization.
- It can utilize "hot expert" caching to mitigate memory bandwidth limitations.

## From [[architectural-strategy-for-stealthy-portable-cli-a|architectural-strategy-for-stealthy-portable-cli-a]] (2026-06-08)
- Local inference engine.
- Can be integrated into the agent for LLM execution.
- Its backend provider can be configured to utilize CPU threads based on passive probes.

## From [[the-sovereign-software-factory-blueprint|the-sovereign-software-factory-blueprint]] (2026-06-08)
- A highly optimized fork is used for a TurboQuant implementation.
- Used in the GPU engine of the Sovereign Node.
- Used via LM Studio.

## From [[drive-research-advanced-local-ai-features-guide|drive-research-advanced-local-ai-features-guide]] (2026-06-08)
- A FOSS engine to watch for native support of 1.58-bit models.

## From [[drive-research-advanced-inference-acceleration|drive-research-advanced-inference-acceleration]] (2026-06-08)
- Used for speculative decoding.
- Supports various compute backends like CUDA, Metal, SYCL, Vulkan, hipBLAS, and OpenBLAS.
- Offers command-line interfaces for neural speculation.
- Provides n-gram based decoding implementations.

## From [[drive-research-agentic-workflows-fastest-best-models|drive-research-agentic-workflows-fastest-best-models]] (2026-06-08)
- Presents an advantage for single-agent workstation deployments maximizing raw generation speed.
- Integrates native Multi-Token Prediction (MTP).
- Demonstrates phenomenal acceleration with MTP speculative decoding support.
- Allows higher-fidelity quantizations without crippling penalties to generation speed.
- Leveraged for deployment in single-agent environments.
- Supports Multi-Token Prediction (MTP).
- Can be used with Qwen3.6.

## From [[drive-research-autonomous-devops-ai-safety-boundaries|drive-research-autonomous-devops-ai-safety-boundaries]] (2026-06-08)
- An inference engine.
- Used for running high-parameter Mixture of Experts models.
- Throughput remains almost perfectly flat under growing load.

## From [[drive-research-cache-optimization-with-ai-chaos-theory|drive-research-cache-optimization-with-ai-chaos-theory]] (2026-06-08)
- An example of an inference process whose threads are pinned to cache-dense cores.
- Receives escalated tasks from the hypervisor.
- Utilizes experimental Tensor Split / Graph Split mode.

## From [[drive-research-cuda-memory-locking-limits-configuration|drive-research-cuda-memory-locking-limits-configuration]] (2026-06-08)
- A popular wrapper for GGML.
- Used for optimizing execution of large language models (LLMs) on local Linux workstations.
- When llama.cpp loads a model, host memory registration is controlled by the environment variable GGML_CUDA_REGISTER_HOST and command-line execution flags such as --mlock and --no-mmap.
- Can be executed within containerized environments like Docker or Podman.
- Can be run as a background service managed by systemd.

## From [[drive-research-du-hast-gesagt-part1|drive-research-du-hast-gesagt-part1]] (2026-06-08)
- Inference engine.
- Powers LM Studio.
- Adding support for 1.58-bit models.
- Used in TurboQuant implementation.
- Used for running local AI models.
- Creates the .GGUF file format.
- Community efforts to integrate ternary kernels.

## From [[drive-research-frankenmoe-blueprint-analysis|drive-research-frankenmoe-blueprint-analysis]] (2026-06-08)
- Used for local bare-metal execution.
- Supports post-training quantization for local deployments.
- Requires post-training quantization for local deployments.
- Can utilize hybrid execution for memory-constrained environments.

## From [[drive-research-hidden-mode-technical-analysis-and-configurati|drive-research-hidden-mode-technical-analysis-and-configurati]] (2026-06-08)
- Enables CPU-GPU hybrid execution for environments where total VRAM is less than the model footprint.
- Implementation of the --cpu-moe flag allows offloading parts of the model to the CPU.
- Used for running GGUF models.

## From [[drive-research-hidden-mode-technical-analysis-and-configuration|drive-research-hidden-mode-technical-analysis-and-configuration]] (2026-06-08)
- Provides CPU-GPU hybrid execution for environments where total VRAM is less than the model footprint.
- Implements the --cpu-moe flag for offloading components.
- Allows routing expert MLP computations to the CPU.

## From [[drive-research-inside-the-pi-coding-agent-optimization-isn|drive-research-inside-the---pi---coding-agent--optimization--isn]] (2026-06-08)
- Can be used to map Pi to a local model stack.
- Bypasses the network for local inference.

## From [[drive-research-llamabench|drive-research-llamabench]] (2026-06-08)
- Bundles llama-bench
- Underlying GGML computational graph is executed by llama-bench

## From [[drive-research-ok-so-designing-a-guide-around-llamabench-would-b|drive-research-ok-so-designing-a-guide-around-llamabench-would-b]] (2026-06-08)
- Can be compiled locally from source for maximum performance.
- Compilation can be optimized with advanced CPU optimization bindings.

## From [[drive-research-optimizing-pi-coding-agent-performance|drive-research-optimizing-pi-coding-agent-performance]] (2026-06-08)
- A local model engine that can be integrated with Pi.
- Configured via models.json.
- Can be used as a local provider.

## From [[drive-research-optimizing-qwen36-on-blackwell-gpus|drive-research-optimizing-qwen36-on-blackwell-gpus]] (2026-06-08)
- A tool that can be used to run multimodal tasks.
- Can be used to bypass the Ollama quantization bug entirely with MTP GGUF checkpoints.
- Requires building with CUDA support.

## From [[drive-research-rust-ecs-cache-optimization-research|drive-research-rust-ecs-cache-optimization-research]] (2026-06-08)
- A runtime used for LLM inference.
- Supports experimental Tensor Split / Graph Split mode.
- Utilizes NVIDIA Collective Communications Library (NCCL) for multi-GPU communication.

## From [[drive-research-speicherbandbreiten-engpass-memory-wall|drive-research-speicherbandbreiten-engpass-memory-wall]] (2026-06-08)
- An open-source community integration of TurboQuant using custom C++ implementations.
- TurboQuant is registered as a new GGML type.
- Developers use specific cache-type flags and pass the draft model via the -md flag.
- The open-source community integrated TurboQuant into it.

## From [[drive-research-token-efficient-bol-processing-architecture|drive-research-token-efficient-bol-processing-architecture]] (2026-06-08)
- An inference engine that has democratized the deployment of localized, task-specific models.
- Used for local quantization with GGUF ecosystems.
- Allows for execution on resource-constrained edge servers, logistics terminals, or local CPU/GPU clusters.

## From [[architectures-and-optimizations-for-speculative-de-micro04|architectures-and-optimizations-for-speculative-de-micro04]] (2026-06-09)
- An optimized, dependency-free C/C++ architecture for LLM inference.
- Supports a multifaceted ecosystem for speculative decoding.
- Enables state-of-the-art inference with minimal setup across diverse hardware.
- Requires models to possess identical tokenizers and special tokens for speculative decoding.

## From [[architectures-and-optimizations-for-speculative-de-micro05|architectures-and-optimizations-for-speculative-de-micro05]] (2026-06-09)
- Used for benchmarking speculative decoding.
- Introduced non-neural, n-gram based speculative decoding implementations.
- Exposes hyperparameters like --draft-max, --draft-min, and --draft-p-min.

## From [[architectures-for-agentic-memory-virtual-context-micro02|architectures-for-agentic-memory-virtual-context-micro02]] (2026-06-09)
- Inference engine that supports constrained decoding frameworks
- Provides the GGML Backus-Naur Form (GBNF) for constrained decoding
- Supports llama-cli, llama-completion, and llama-server interfaces

## From [[drive-research-32gb-vram-ai-reasoning-models-micro01|drive-research-32gb-vram-ai-reasoning-models-micro01]] (2026-06-09)
- Backend that uses GGUF format
- Used by platforms like Ollama and LM Studio

## From [[drive-research-32gb-vram-ai-reasoning-models-micro02|drive-research-32gb-vram-ai-reasoning-models-micro02]] (2026-06-09)
- Is incredibly accessible.
- Natively supporting the ubiquitous GGUF format.
- Primary advantage is its ability to seamlessly offload layers across the PCIe bus to system RAM.
- Highly unsuitable for rapid reasoning tasks where prompt ingestion (TTFT) and token generation speed are critical parameters for success.

## From [[drive-research-architecting-zero-configuration-portable-agents-s-micro03|drive-research-architecting-zero-configuration-portable-agents-s-micro03]] (2026-06-09)
- An example of a local inference engine.
- Agent's internal logic configures the runtime backend provider to utilize CPU threads.

## From [[drive-research-benchmarking-llamacpp-server-prefill-tokens-micro02|drive-research-benchmarking-llamacpp-server-prefill-tokens-micro02]] (2026-06-09)
- The document discusses optimizations for llama.cpp server.
- References to llama.cpp appear frequently in the 'Referenzen' section.
- It is a project related to LLM inference.

## From [[drive-research-linux-gaming-and-ai-build-guide-micro03|drive-research-linux-gaming-and-ai-build-guide-micro03]] (2026-06-09)
- VRAM Requirements Guide for Local LLMs in 2026

## From [[drive-research-linux-gaming-and-ai-build-guide-micro05|drive-research-linux-gaming-and-ai-build-guide-micro05]] (2026-06-09)
- Inference framework that offloads transformer layers to system RAM when VRAM is exceeded.
- Severely cripples inference speeds and latency when offloading.

## From [[drive-research-llama-bench-performance-benchmarking-tool-micro01|drive-research-llama-bench-performance-benchmarking-tool-micro01]] (2026-06-09)
- Core repository that bundles the llama-bench tool.
- GGML computational graph is executed against target hardware backend APIs via llama-bench.

## From [[drive-research-llamacpp-optimization-blueprint-micro01|drive-research-llamacpp-optimization-blueprint-micro01]] (2026-06-09)
- Optimization is being done for llama.cpp.
- It is part of the 'Drive Research Llama.cpp Optimization Blueprint'.

## From [[drive-research-llamacpp-optimization-blueprint-micro03|drive-research-llamacpp-optimization-blueprint-micro03]] (2026-06-09)
- A bare-metal environment for achieving maximum throughput.
- Supports continuous batching and parallel decoding.
- Integrates TurboQuant algorithms into its CUDA backend.

## From [[drive-research-llm-inference-engine-audit-2026-micro02|drive-research-llm-inference-engine-audit-2026-micro02]] (2026-06-09)
- Universal implementation.
- Written purely in C and C++.
- Operates with zero third-party dependencies and virtually zero memory allocation during runtime.
- Serves as the bedrock of local open-source inference, prioritizing maximum compatibility across any hardware profile.
- Requires meticulous manual parameter tuning.
- Functions primarily as a single-process runtime.
- Remains the gold standard for granular control in resource-constrained, heterogeneous environments.
- Bedrock of open-source artificial intelligence.
- Sacrifices user-friendly automation for absolute, ubiquitous control.
- Ensures that models will compile and execute flawlessly on any heterogeneous hardware environment.
- Serves as the universal fallback and the foundation for most edge AI deployments.

## From [[drive-research-llm-inference-engine-audit-2026-micro03|drive-research-llm-inference-engine-audit-2026-micro03]] (2026-06-09)
- Compared against MLX, Ollama, and vLLM for local AI inference.
- Can be run on Intel® GPUs.
- OpenVINO 2026.1 has a backend for it.
- Support for Intel Neural Processing Unit (NPU) and Intel Arc GPU acceleration in it is discussed on GitHub.

## From [[drive-research-optimizing-cuda-performance-with-fp4-fp6-micro01|drive-research-optimizing-cuda-performance-with-fp4-fp6-micro01]] (2026-06-09)
- A runtime library that experiences a Matrix Multiply Quantized (MMQ) kernel crash under CUDA 13.1 on Blackwell.
- The MMQ kernel crash triggers segmentation faults, forcing a fallback to standard cuBLAS routines.
- When FORCE_CUBLAS=ON is set, the high-speed MMQ path is disabled entirely.

## From [[optimizing-nvidia-blackwell-sm120-part3-micro06|optimizing-nvidia-blackwell-sm120-part3-micro06]] (2026-06-09)
- It is a project that includes the llama-bench tool.
- It has a CUDA backend.
- It is used for GGUF models.

## From [[resilient-rust-based-mcp-client-and-llm-orchestrat-micro07|resilient-rust-based-mcp-client-and-llm-orchestrat-micro07]] (2026-06-09)
- Parser implementation is robust enough to fool enterprise orchestration frameworks.
- Tool calling integration capabilities and operational overhead examined.
- Community-driven project.
- Framework provides exhaustive, production-ready support for OpenAI-compatible function and tool calling.
- Active maintenance ensures compatibility with major downstream orchestration tools.

## From [[the-architecture-of-speculative-decoding-and-infer-part1-micro01|the-architecture-of-speculative-decoding-and-infer-part1-micro01]] (2026-06-09)
- A system that implements speculative decoding.
- Offers special configurations to bypass memory bottlenecks.
- Provides n-gram-based decoding implementations as an alternative to neural draft models.
- Requires compilation with specific hardware-specific backends for maximum performance.

## From [[the-architecture-of-speculative-decoding-and-infer-part1-micro05|the-architecture-of-speculative-decoding-and-infer-part1-micro05]] (2026-06-09)
- A framework mentioned for implementing TurboQuant and speculative decoding systems.

## From [[the-architecture-of-speculative-decoding-and-infer-part1-micro06|the-architecture-of-speculative-decoding-and-infer-part1-micro06]] (2026-06-09)
- Open-source community has successfully integrated TurboQuant into llama.cpp via custom C++ implementations
- Used for edge deployments on Apple Silicon utilizing Metal GPU kernels or consumer NVIDIA GPUs
- TurboQuant is registered as a new GGML type (e.g., GGML_TYPE_TQ3 and GGML_TYPE_TQ4)
- Framework intercepts the FP16 or FP32 K/V vectors during the cache write phase
- Developers executing speculative decoding with TurboQuant on llama.cpp utilize specific cache-type flags (-ctk and -ctv) while passing the draft model via the -md flag
- Advanced forks have introduced critical stabilization features beyond the original Google paper

## From [[ultimate-local-ai-development-stack-for-vscodium-micro03|ultimate-local-ai-development-stack-for-vscodium-micro03]] (2026-06-09)
- An inference engine that supports 1.58-bit models via LM Studio.

## From [[drive-research-research-process-steps-micro02|drive-research-research-process-steps-micro02]] (2026-06-10)
- An inference framework that introduced a major breaking change on June 12, 2024.
- Uses the ggml library for low-level operations.
- Uses processor-specific vector extensions to optimize dot product math.

## From [[drive-research-research-process-steps-micro03|drive-research-research-process-steps-micro03]] (2026-06-10)
- Uses the llama_kv_cache_get_padding() function to allocate cache blocks in padded chunks.
- Uses ggml_view_3d and ggml_view_2d functions to manage blocks.

## From [[openclaw-deep-research-part8-micro02|openclaw-deep-research-part8-micro02]] (2026-06-10)
- Supports local model hosting

## From [[optimizing-nvidia-blackwell-sm120-part1-micro01|optimizing-nvidia-blackwell-sm120-part1-micro01]] (2026-06-10)
- An inference engine built upon the ggml tensor library.
- Prioritizes bare-metal resource saturation and minimal software abstraction.
- Supports multi-GPU orchestration and probabilistic sampling.

## From [[optimizing-nvidia-blackwell-sm120-part1-micro06|optimizing-nvidia-blackwell-sm120-part1-micro06]] (2026-06-10)
- Runtime library experiencing MMQ kernel crashes under CUDA 13.1.
- Achieves high throughput (201–211 tok/s) on RTX 5090 with stable drivers.

## From [[optimizing-nvidia-blackwell-sm120-part2-micro04|optimizing-nvidia-blackwell-sm120-part2-micro04]] (2026-06-10)
- Core repository containing llama-bench.
- Includes performance patches like PR #19625 and PR #20551.

## From [[optimizing-nvidia-blackwell-sm120-part2-micro05|optimizing-nvidia-blackwell-sm120-part2-micro05]] (2026-06-10)
- LLM inference in C/C++
- Supports various backends including CUDA, Vulkan, ROCm, and SYCL

## From [[optimizing-nvidia-blackwell-sm120-part3-micro01|optimizing-nvidia-blackwell-sm120-part3-micro01]] (2026-06-10)
- Inference framework that introduced a major breaking change on June 12, 2024.
- Uses the ggml library for low-level operations.
- Includes a variety of binaries for model execution and processing.

## From [[optimizing-nvidia-blackwell-sm120-part3-micro02|optimizing-nvidia-blackwell-sm120-part3-micro02]] (2026-06-10)
- Uses llama_kv_cache_get_padding() to allocate cache blocks
- Uses ggml_view_3d and ggml_view_2d functions for block management

## From [[optimizing-nvidia-blackwell-sm120-part3-micro03|optimizing-nvidia-blackwell-sm120-part3-micro03]] (2026-06-10)
- Execution architecture must adapt to hardware accelerators ranging from consumer-grade to enterprise-grade GPU clusters.
- Utilizes an automated parameter-fitting subsystem to reduce user configuration errors.
- Requires matching data structures of host operating systems for cross-platform portability.

## From [[phantom-drive-autonomous-llm-deployment-architect-micro02|phantom-drive-autonomous-llm-deployment-architect-micro02]] (2026-06-10)
- An ecosystem containing quantization formats like Q4_K_M and Q5_K_M.
- Utilizes memory-mapped files (mmap) by default for loading model weights.

## From [[resilient-rust-based-mcp-client-and-llm-orchestrat-micro05|resilient-rust-based-mcp-client-and-llm-orchestrat-micro05]] (2026-06-10)
- An inference engine for local Large Language Model (LLM) inference.
- Includes a llama-server HTTP component.
- Supports OpenAI-compatible tool calling via its chat completions endpoint.

## From [[the-agentic-operating-environment-a-synthesis-arc-micro01|the-agentic-operating-environment-a-synthesis-arc-micro01]] (2026-06-10)
- An inference engine that can be spun up via Docker/Podman.

## From [[the-architecture-of-speculative-decoding-and-infer-part1-micro03|the-architecture-of-speculative-decoding-and-infer-part1-micro03]] (2026-06-10)
- Supports the GGUF format.
- Capable of offloading layers across the PCIe bus to system RAM.
