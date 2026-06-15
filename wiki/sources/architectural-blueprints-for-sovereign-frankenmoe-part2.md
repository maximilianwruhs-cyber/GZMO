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
- [[deepseek-ai-deepseek-r1-distill-qwen-7b|deepseek-ai/DeepSeek-R1-Distill-Qwen-7B]] (MODEL)
- [[proxmox-egpu-hypervisor|Proxmox eGPU hypervisor]] (SYSTEM)
- [[sovereign-moe-yaml|sovereign-moe.yaml]] (CONCEPT)
- [[layer-splitting-mode|Layer-Splitting Mode]] (CONCEPT)
- [[sovereign-frankenmoe|Sovereign FrankenMoE]] (CONCEPT)
- [[llama-server|llama-server]] (TOOL)
- [[kvm-vm-101-core-database-ai|KVM VM 101: Core Database AI]] (SYSTEM)
- [[qwen-qwen2-5-coder-7b-instruct|Qwen/Qwen2.5-Coder-7B-Instruct]] (MODEL)
- [[proxmox-ve-hypervisor|Proxmox VE HYPERVISOR]] (SYSTEM)
- [[qwen2-moe|qwen2_moe]] (CONCEPT)

## Relations
- Sovereign FrankenMoE → USES → Proxmox eGPU hypervisor
- Sovereign FrankenMoE → USES → Qwen/Qwen2.5-Coder-7B-Instruct
- Sovereign FrankenMoE → USES → deepseek-ai/DeepSeek-R1-Distill-Qwen-7B
- sovereign-moe.yaml → USES → Qwen/Qwen2.5-Coder-7B-Instruct
- sovereign-moe.yaml → USES → qwen2_moe
- Proxmox VE HYPERVISOR → USES → KVM VM 101: Core Database AI
- KVM VM 101: Core Database AI → USES → llama-server
- llama-server → USES → Layer-Splitting Mode
