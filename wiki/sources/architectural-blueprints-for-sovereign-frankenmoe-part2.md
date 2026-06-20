---
type: source
title: architectural-blueprints-for-sovereign-frankenmoe-part2
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# architectural-blueprints-for-sovereign-frankenmoe-part2

Ingested source summary (2026-06-08).

## Entities
- [deepseek-ai/DeepSeek-R1-Distill-Qwen-7B](/entities/deepseek-ai-deepseek-r1-distill-qwen-7b.md) (MODEL)
- [Proxmox eGPU hypervisor](/entities/proxmox-egpu-hypervisor.md) (SYSTEM)
- [sovereign-moe.yaml](/entities/sovereign-moe-yaml.md) (CONCEPT)
- [Layer-Splitting Mode](/entities/layer-splitting-mode.md) (CONCEPT)
- [Sovereign FrankenMoE](/entities/sovereign-frankenmoe.md) (CONCEPT)
- [llama-server](/entities/llama-server.md) (TOOL)
- [KVM VM 101: Core Database AI](/entities/kvm-vm-101-core-database-ai.md) (SYSTEM)
- [Qwen/Qwen2.5-Coder-7B-Instruct](/entities/qwen-qwen2-5-coder-7b-instruct.md) (MODEL)
- [Proxmox VE HYPERVISOR](/entities/proxmox-ve-hypervisor.md) (SYSTEM)
- [qwen2_moe](/entities/qwen2-moe.md) (CONCEPT)

## Relations
- Sovereign FrankenMoE → USES → Proxmox eGPU hypervisor
- Sovereign FrankenMoE → USES → Qwen/Qwen2.5-Coder-7B-Instruct
- Sovereign FrankenMoE → USES → deepseek-ai/DeepSeek-R1-Distill-Qwen-7B
- sovereign-moe.yaml → USES → Qwen/Qwen2.5-Coder-7B-Instruct
- sovereign-moe.yaml → USES → qwen2_moe
- Proxmox VE HYPERVISOR → USES → KVM VM 101: Core Database AI
- KVM VM 101: Core Database AI → USES → llama-server
- llama-server → USES → Layer-Splitting Mode
