---
type: entity
title: Docker
created: 2026-06-08
updated: 2026-06-10
sources: 46
tags: []
status: draft
gzmo_synthetic: true
---














































# Docker

Type: TOOL

## From [[ai-research-part7|ai-research-part7]] (2026-06-08)
- It is used for deeply sandboxed containers by openclaw-contained.
- Agents running as the root user inside unhardened Docker containers were a root cause of privilege escalation.

## From [[architecting-the-minimalist-linux-desktop-a-compa-part1|architecting-the-minimalist-linux-desktop-a-compa-part1]] (2026-06-08)
- Alpine Linux is the default choice for deploying microservices and applications in this environment.
- A container runtime that heavily utilizes OverlayFS's stacked lower layers via the overlay2 storage driver.
- Docker's overlay2 driver supports up to 128 layers.

## From [[openclaw-deep-research-part2|openclaw-deep-research-part2]] (2026-06-08)
- OpenClaw can be set up in a Docker with limited permissions and an inability to escalate privileges.

## From [[the-architecture-of-scientific-inquiry-and-academi|the-architecture-of-scientific-inquiry-and-academi]] (2026-06-08)
- Used for advanced reproducibility setups.
- Encapsulates exact software, OS-level dependencies, and libraries.
- Ensures true computational portability.

## From [[the-openclaw-architecture-and-tri-circuit-autonomo-part2|the-openclaw-architecture-and-tri-circuit-autonomo-part2]] (2026-06-08)
- Used for sandboxing in Circuit III (OpenHands).
- Socket permissions may restrict access.
- Provides isolation for agents.

## From [[the-sovereign-software-factory-blueprint|the-sovereign-software-factory-blueprint]] (2026-06-08)
- Used in the Linux Kernel / Sandbox.
- Agents write and test code strictly inside disposable DevContainers.

## From [[drive-research-cuda-memory-locking-limits-configuration|drive-research-cuda-memory-locking-limits-configuration]] (2026-06-08)
- A container runtime environment where llama.cpp or vLLM can be executed.
- Container runtime drops most Linux capabilities by default.
- Processes run with highly restrictive default memory-locking limits under standard configurations.
- Containers must be launched with the flag --ulimit memlock=-1 or be granted the explicit capability CAP_IPC_LOCK via --cap-add=IPC_LOCK to circumvent restrictive limits.

## From [[drive-research-die-architektur-der-wissenschaftlichen-forschung|drive-research-die-architektur-der-wissenschaftlichen-forschung]] (2026-06-08)
- A containerization platform.
- Used for advanced reproducibility setups.
- Encapsulates exact software, OS-level dependencies, and libraries.

## From [[drive-research-du-hast-gesagt-part1|drive-research-du-hast-gesagt-part1]] (2026-06-08)
- Used for DevContainers.
- Provides containerization.
- Used for Agent Sandboxing.
- Prevents autonomous AI from running terminal commands directly on the host OS.
- Installed via `apt install -y docker.io docker-compose-v2`.
- Used to run tests in a sandboxed environment.
- Prevents damage to the host OS.

## From [[drive-research-enhancing-local-ai-hypervisor-architecture|drive-research-enhancing-local-ai-hypervisor-architecture]] (2026-06-08)
- Containerized stacks can be run inside LXC containers.
- Requires NVIDIA Container Toolkit for GPU passthrough.
- Its cgroup management can conflict with LXC.
- Containerized stacks can run within LXC.
- Requires NVIDIA Container Toolkit for GPU access.

## From [[drive-research-hermes-compression-and-bol-architecture|drive-research-hermes-compression-and-bol-architecture]] (2026-06-08)
- backend for Hermes framework execution

## From [[drive-research-mcp-landscape-research-report|drive-research-mcp-landscape-research-report]] (2026-06-08)
- An ephemeral, containerized sandbox that organizations must leverage.

## From [[drive-research-pi-coding-agent-local-deployment-customization|drive-research-pi-coding-agent-local-deployment-customization]] (2026-06-08)
- An isolated execution container that Pi can be deployed inside.
- Requires local inference server to bind to all network interfaces.
- Can be used to execute Pi inside isolated virtual environments.

## From [[drive-research-welcome-to-the-master-assembly-manual-for-the-sove|drive-research-welcome-to-the-master-assembly-manual-for-the-sove]] (2026-06-08)
- Used for sandboxing AI.
- Is installed on Ubuntu.
- docker.io and docker-compose-v2 are specific packages.

## From [[gzmo-soul-merged-new-part1|gzmo-soul-merged-new-part1]] (2026-06-09)
- Bietet Container-basierte Abschottung
- Kernel wird mit dem Host-Betriebssystem geteilt
- Erhöht das Risiko für Container Escapes

## From [[building-a-private-local-ai-development-environmen-micro06|building-a-private-local-ai-development-environmen-micro06]] (2026-06-09)
- Used for DevContainers.
- Provides containerization for sandboxing.

## From [[drive-research-advanced-typescript-execution-in-the-bun-runtime-micro03|drive-research-advanced-typescript-execution-in-the-bun-runtime-micro03]] (2026-06-09)
- Furthermore, deploying Bun within Docker significantly increases container size; the official oven/bun image expands container footprints to roughly 450MB, compared to Node.js's 180MB and Deno's highly optimized 73MB slim images.

## From [[drive-research-architektur-und-implementierung-intelligenter-ki-a-micro02|drive-research-architektur-und-implementierung-intelligenter-ki-a-micro02]] (2026-06-09)
- Containers share the kernel with the host operating system.
- Increases the risk of 'Container Escapes'.

## From [[drive-research-bun-file-parsing-dependency-shortlist-micro02|drive-research-bun-file-parsing-dependency-shortlist-micro02]] (2026-06-09)
- Image build times are increased by large dependencies.

## From [[drive-research-bun-typescript-performance-tips-micro03|drive-research-bun-typescript-performance-tips-micro03]] (2026-06-09)
- Deploying Bun within Docker significantly increases container size.

## From [[drive-research-linux-gaming-and-ai-build-guide-micro02|drive-research-linux-gaming-and-ai-build-guide-micro02]] (2026-06-09)
- Historical industry standard for containerization.
- Requires a persistent, privileged dockerd background daemon running as root.
- Containers initialize in 1.2 seconds.

## From [[drive-research-linux-gaming-and-ai-build-guide-micro03|drive-research-linux-gaming-and-ai-build-guide-micro03]] (2026-06-09)
- Compared against Podman for AI Infrastructure in 2026
- Compared against Podman in 2026 Migration Guide

## From [[drive-research-linux-gaming-and-ai-build-guide-micro07|drive-research-linux-gaming-and-ai-build-guide-micro07]] (2026-06-09)
- Comparison with Podman for AI Infrastructure
- Migration Guide

## From [[drive-research-llm-inference-engine-audit-2026-micro02|drive-research-llm-inference-engine-audit-2026-micro02]] (2026-06-09)
- Model management philosophy emulated by Ollama.

## From [[drive-research-scientific-writing-and-publication-process-micro02|drive-research-scientific-writing-and-publication-process-micro02]] (2026-06-09)
- An advanced reproducibility setup.
- Encapsulates exact software, OS-level dependencies, and libraries.

## From [[openclaw-deep-research-part1-micro02|openclaw-deep-research-part1-micro02]] (2026-06-09)
- Recommended for isolating OpenClaw environments.
- NanoClaw enforces OS-level container isolation.

## From [[openclaw-deep-research-part1-micro03|openclaw-deep-research-part1-micro03]] (2026-06-09)
- Used to run agent sessions in isolated containers.
- The sandbox-setup.sh script is used to build the sandbox image.

## From [[openclaw-deep-research-part8-micro04|openclaw-deep-research-part8-micro04]] (2026-06-09)
- Used for containerizing agents for tenant separation in AutoGen.

## From [[phantom-drive-autonomous-llm-deployment-architect-micro01|phantom-drive-autonomous-llm-deployment-architect-micro01]] (2026-06-09)
- Used to build the llama-server binary.
- A multi-stage Docker build pipeline is mandatory.
- Used to create a temporary container for extraction.

## From [[resilient-rust-based-mcp-client-and-llm-orchestrat-micro03|resilient-rust-based-mcp-client-and-llm-orchestrat-micro03]] (2026-06-09)
- Daemon is instructed to automatically inject and mount a tiny, dedicated init process.
- Daemon pushes bytes down the TCP socket.
- Daemon automatically terminates the log stream connection upon container exit.

## From [[resilient-rust-based-mcp-client-and-llm-orchestrat-micro07|resilient-rust-based-mcp-client-and-llm-orchestrat-micro07]] (2026-06-09)
- Used for sandboxing code execution.

## From [[the-2026-linux-workstation-micro03|the-2026-linux-workstation-micro03]] (2026-06-09)
- Historical industry standard for containerization.
- Requires a persistent, privileged background daemon.
- Containers initialize slower and consume more memory than Podman.

## From [[the-cognitive-architecture-of-openclaw-agents-micro04|the-cognitive-architecture-of-openclaw-agents-micro04]] (2026-06-09)
- Used for OS-level sandboxing.
- Containers are spun up using the bollard crate.

## From [[the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro02|the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro02]] (2026-06-09)
- Used for sandboxing tool execution.

## From [[the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro08|the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro08]] (2026-06-09)
- Used to create sandboxes like OpenHands.
- Enables isolated testing environments.

## From [[the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro09|the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro09]] (2026-06-09)
- Used for managing high-privilege sandboxes for code execution.
- OpenHands Docker verification is part of Circuit III.

## From [[drive-research-linux-gaming-and-ai-build-guide-micro06|drive-research-linux-gaming-and-ai-build-guide-micro06]] (2026-06-10)
- Requires a persistent, privileged dockerd background daemon

## From [[openclaw-deep-research-part7-micro01|openclaw-deep-research-part7-micro01]] (2026-06-10)
- Supported install method for deploying OpenClaw.

## From [[openclaw-deep-research-part8-micro02|openclaw-deep-research-part8-micro02]] (2026-06-10)
- Used for self-hosted deployments of CrewAI

## From [[openclaw-deep-research-part9-micro02|openclaw-deep-research-part9-micro02]] (2026-06-10)
- Used for sandboxing tool execution

## From [[openclaw-deep-research-part9-micro05|openclaw-deep-research-part9-micro05]] (2026-06-10)
- Used to run agent sessions in isolated containers

## From [[openclaw-part1-micro03|openclaw-part1-micro03]] (2026-06-10)
- An environment where the Gateway can be deployed with isolated execution risk.

## From [[prompt-agent-engineering-part6-micro01|prompt-agent-engineering-part6-micro01]] (2026-06-10)
- Used to containerize components like the Orchestrator and MCP server.

## From [[resilient-rust-based-mcp-client-and-llm-orchestrat-micro02|resilient-rust-based-mcp-client-and-llm-orchestrat-micro02]] (2026-06-10)
- Container orchestration platform.
- Provides an Engine API that Bollard interacts with via low-level client.

## From [[the-2026-linux-workstation-micro04|the-2026-linux-workstation-micro04]] (2026-06-10)
- Compared to Podman regarding speed and weight

## From [[the-agentic-operating-environment-a-synthesis-arc-micro01|the-agentic-operating-environment-a-synthesis-arc-micro01]] (2026-06-10)
- Can be detected by GZMO to spin up tools.
