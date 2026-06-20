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
- [64GB of DDR5-6000 CL30 memory](/entities/64gb-of-ddr5-6000-cl30-memory.md) (TOOL)
- [rpm-ostree](/entities/rpm-ostree.md) (TOOL)
- [WoW64 mode](/entities/wow64-mode.md) (CONCEPT)
- [the-2026-linux-workstation](/entities/the-2026-linux-workstation.md) (SYSTEM)
- [Intel's Nova Lake](/entities/intel-s-nova-lake.md) (CONCEPT)
- [Nix shells](/entities/nix-shells.md) (TOOL)
- [Docker](/entities/docker.md) (TOOL)
- [NVIDIA Container Toolkit](/entities/nvidia-container-toolkit.md) (ORGANIZATION)
- [ROCm 7.1](/entities/rocm-7-1.md) (TOOL)
- [MuQSS](/entities/muqss.md) (CONCEPT)
- [Liquorix Kernel](/entities/liquorix-kernel.md) (SYSTEM)
- [UALink](/entities/ualink.md) (CONCEPT)
- [CXL](/entities/cxl.md) (CONCEPT)
- [artificial intelligence](/entities/artificial-intelligence.md) (CONCEPT)
- [Wine 11.0](/entities/wine-11-0.md) (TOOL)
- [TensorRT-LLM](/entities/tensorrt-llm.md) (TOOL)
- [Completely Fair Scheduler (CFS)](/entities/completely-fair-scheduler-cfs.md) (CONCEPT)
- [Lutris](/entities/lutris.md) (TOOL)
- [CUDA Toolkit](/entities/cuda-toolkit.md) (TOOL)
- [DXVK](/entities/dxvk.md) (TOOL)
- [Zen 5](/entities/zen-5.md) (CONCEPT)
- [Zen 6](/entities/zen-6.md) (CONCEPT)
- [Wayland](/entities/wayland.md) (SYSTEM)
- [RTX 5090](/entities/rtx-5090.md) (TOOL)
- [PCIe 5.0 NVMe storage topology](/entities/pcie-5-0-nvme-storage-topology.md) (CONCEPT)
- [Pop!_OS](/entities/pop-os.md) (SYSTEM)
- [ATX 3.1 12V-2x6 standard](/entities/atx-3-1-12v-2x6-standard.md) (CONCEPT)
- [Fedora Kinoite](/entities/fedora-kinoite.md) (SYSTEM)
- [Samsung 990 Pro (4TB)](/entities/samsung-990-pro-4tb.md) (TOOL)
- [Bazzite](/entities/bazzite.md) (SYSTEM)
- [NTsync](/entities/ntsync.md) (CONCEPT)
- [FlashAttention 3](/entities/flashattention-3.md) (TOOL)
- [CUDA 13.0](/entities/cuda-13-0.md) (TOOL)
- [Crucial T705 (4TB)](/entities/crucial-t705-4tb.md) (TOOL)
- [TKG Kernel](/entities/tkg-kernel.md) (SYSTEM)
- [NVMe RAID 0](/entities/nvme-raid-0.md) (CONCEPT)
- [Ryzen 9 9950X3D](/entities/ryzen-9-9950x3d.md) (TOOL)
- [Proton 11](/entities/proton-11.md) (TOOL)
- [PyTorch](/entities/pytorch.md) (TOOL)
- [Nix flakes](/entities/nix-flakes.md) (TOOL)
- [ProtonGE](/entities/protonge.md) (TOOL)
- [MangoHUD](/entities/mangohud.md) (TOOL)
- [AM5 socket](/entities/am5-socket.md) (CONCEPT)
- [AMD](/entities/amd.md) (ORGANIZATION)
- [WD_Black SN8100](/entities/wd-black-sn8100.md) (TOOL)
- [PDS (Project C)](/entities/pds-project-c.md) (CONCEPT)
- [cuDNN](/entities/cudnn.md) (TOOL)
- [SteamOS](/entities/steamos.md) (SYSTEM)
- [PCIe 6.0](/entities/pcie-6-0.md) (CONCEPT)
- [Podman](/entities/podman.md) (TOOL)
- [vkBasalt](/entities/vkbasalt.md) (TOOL)

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
