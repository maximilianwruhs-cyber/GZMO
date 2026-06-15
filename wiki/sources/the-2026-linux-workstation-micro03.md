---
type: source
title: the-2026-linux-workstation-micro03
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# the-2026-linux-workstation-micro03

Ingested source summary (2026-06-09).

## Entities
- [[64gb-of-ddr5-6000-cl30-memory|64GB of DDR5-6000 CL30 memory]] (TOOL)
- [[rpm-ostree|rpm-ostree]] (TOOL)
- [[wow64-mode|WoW64 mode]] (CONCEPT)
- [[the-2026-linux-workstation|the-2026-linux-workstation]] (SYSTEM)
- [[intel-s-nova-lake|Intel's Nova Lake]] (CONCEPT)
- [[nix-shells|Nix shells]] (TOOL)
- [[docker|Docker]] (TOOL)
- [[nvidia-container-toolkit|NVIDIA Container Toolkit]] (ORGANIZATION)
- [[rocm-7-1|ROCm 7.1]] (TOOL)
- [[muqss|MuQSS]] (CONCEPT)
- [[liquorix-kernel|Liquorix Kernel]] (SYSTEM)
- [[ualink|UALink]] (CONCEPT)
- [[cxl|CXL]] (CONCEPT)
- [[artificial-intelligence|artificial intelligence]] (CONCEPT)
- [[wine-11-0|Wine 11.0]] (TOOL)
- [[tensorrt-llm|TensorRT-LLM]] (TOOL)
- [[completely-fair-scheduler-cfs|Completely Fair Scheduler (CFS)]] (CONCEPT)
- [[lutris|Lutris]] (TOOL)
- [[cuda-toolkit|CUDA Toolkit]] (TOOL)
- [[dxvk|DXVK]] (TOOL)
- [[zen-5|Zen 5]] (CONCEPT)
- [[zen-6|Zen 6]] (CONCEPT)
- [[wayland|Wayland]] (SYSTEM)
- [[rtx-5090|RTX 5090]] (TOOL)
- [[pcie-5-0-nvme-storage-topology|PCIe 5.0 NVMe storage topology]] (CONCEPT)
- [[pop-os|Pop!_OS]] (SYSTEM)
- [[atx-3-1-12v-2x6-standard|ATX 3.1 12V-2x6 standard]] (CONCEPT)
- [[fedora-kinoite|Fedora Kinoite]] (SYSTEM)
- [[samsung-990-pro-4tb|Samsung 990 Pro (4TB)]] (TOOL)
- [[bazzite|Bazzite]] (SYSTEM)
- [[ntsync|NTsync]] (CONCEPT)
- [[flashattention-3|FlashAttention 3]] (TOOL)
- [[cuda-13-0|CUDA 13.0]] (TOOL)
- [[crucial-t705-4tb|Crucial T705 (4TB)]] (TOOL)
- [[tkg-kernel|TKG Kernel]] (SYSTEM)
- [[nvme-raid-0|NVMe RAID 0]] (CONCEPT)
- [[ryzen-9-9950x3d|Ryzen 9 9950X3D]] (TOOL)
- [[proton-11|Proton 11]] (TOOL)
- [[pytorch|PyTorch]] (TOOL)
- [[nix-flakes|Nix flakes]] (TOOL)
- [[protonge|ProtonGE]] (TOOL)
- [[mangohud|MangoHUD]] (TOOL)
- [[am5-socket|AM5 socket]] (CONCEPT)
- [[amd|AMD]] (ORGANIZATION)
- [[wd-black-sn8100|WD_Black SN8100]] (TOOL)
- [[pds-project-c|PDS (Project C)]] (CONCEPT)
- [[cudnn|cuDNN]] (TOOL)
- [[steamos|SteamOS]] (SYSTEM)
- [[pcie-6-0|PCIe 6.0]] (CONCEPT)
- [[podman|Podman]] (TOOL)
- [[vkbasalt|vkBasalt]] (TOOL)

## Relations
- the-2026-linux-workstation → USES → NVIDIA Container Toolkit
- the-2026-linux-workstation → USES → Proton 11
- the-2026-linux-workstation → USES → Podman
- the-2026-linux-workstation → USES → Nix shells
- the-2026-linux-workstation → USES → Bazzite
- the-2026-linux-workstation → USES → Liquorix Kernel
- the-2026-linux-workstation → USES → Nix flakes
- the-2026-linux-workstation → USES → RTX 5090
- the-2026-linux-workstation → USES → Ryzen 9 9950X3D
- the-2026-linux-workstation → USES → 64GB of DDR5-6000 CL30 memory
- the-2026-linux-workstation → USES → PCIe 5.0 NVMe storage topology
- Crucial T705 (4TB) → PART_OF → PCIe 5.0 NVMe storage topology
- WD_Black SN8100 → PART_OF → PCIe 5.0 NVMe storage topology
- the-2026-linux-workstation → USES → Completely Fair Scheduler (CFS)
- the-2026-linux-workstation → USES → Wayland
- Proton 11 → USES → DXVK
- Proton 11 → PART_OF → Wine 11.0
- Proton 11 → USES → NTsync
- Podman → USES → NVIDIA Container Toolkit
- Bazzite → PART_OF → Fedora Kinoite
- Bazzite → USES → rpm-ostree
- Bazzite → USES → ProtonGE
- Bazzite → USES → MangoHUD
- Bazzite → USES → vkBasalt
- Bazzite → USES → NVIDIA Container Toolkit
- Fedora Kinoite → USES → rpm-ostree
- Liquorix Kernel → USES → PDS (Project C)
- Liquorix Kernel → USES → MuQSS
- Wine 11.0 → USES → WoW64 mode
- Lutris → USES → ProtonGE
- CUDA Toolkit → USES → Podman
- cuDNN → USES → Podman
- PyTorch → USES → Podman
- PyTorch → USES → Nix flakes
- Docker → USES → the-2026-linux-workstation
- Nix flakes → USES → PyTorch
- NVIDIA Container Toolkit → USES → Podman
- ATX 3.1 12V-2x6 standard → RELATED_TO → RTX 5090
- RTX 5090 → PART_OF → PCIe 5.0 NVMe storage topology
- RTX 5090 → USES → ATX 3.1 12V-2x6 standard
- PCIe 5.0 NVMe storage topology → PART_OF → RTX 5090
- PCIe 6.0 → PART_OF → UALink
- PCIe 6.0 → PART_OF → CXL
- AM5 socket → PART_OF → Ryzen 9 9950X3D
- Ryzen 9 9950X3D → PART_OF → Zen 5
- Ryzen 9 9950X3D → RELATED_TO → RTX 5090
