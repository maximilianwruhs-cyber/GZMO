---
type: source
title: drive-research-analyze-the-pdf-to-create-a-step-by-step-guide-of
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# drive-research-analyze-the-pdf-to-create-a-step-by-step-guide-of

Ingested source summary (2026-06-08).

## Entities
- [Linux scheduler](/entities/linux-scheduler.md) (SYSTEM)
- [GSP firmware](/entities/gsp-firmware.md) (CONCEPT)
- [Host Memory Buffer (HMB)](/entities/host-memory-buffer-hmb.md) (CONCEPT)
- [BBR](/entities/bbr.md) (CONCEPT)
- [NVIDIA driver](/entities/nvidia-driver.md) (TOOL)
- [PRIME](/entities/prime.md) (CONCEPT)
- [realtek-r8126-dkms](/entities/realtek-r8126-dkms.md) (TOOL)
- [NVIDIA GeForce RTX 5070 Ti](/entities/nvidia-geforce-rtx-5070-ti.md) (HARDWARE)
- [Curve Optimizer](/entities/curve-optimizer.md) (CONCEPT)
- [AMI UEFI](/entities/ami-uefi.md) (SYSTEM)
- [APST](/entities/apst.md) (CONCEPT)
- [amd-pstate-epp driver](/entities/amd-pstate-epp-driver.md) (SYSTEM)
- [NVML](/entities/nvml.md) (TOOL)
- [Wayland](/entities/wayland.md) (SYSTEM)
- [Realtek RTL8126](/entities/realtek-rtl8126.md) (HARDWARE)
- [ASRock X870E Taichi Lite User Manual](/entities/asrock-x870e-taichi-lite-user-manual.md) (BOOK)
- [nvidia-persistenced](/entities/nvidia-persistenced.md) (SYSTEM)
- [Ubuntu Extreme Hardware Tuning](/entities/ubuntu-extreme-hardware-tuning.md) (SYSTEM)
- [ethtool](/entities/ethtool.md) (TOOL)
- [sysctl](/entities/sysctl.md) (TOOL)
- [KIOXIA Exceria Plus G4](/entities/kioxia-exceria-plus-g4.md) (HARDWARE)
- [Energy Performance Preference (EPP)](/entities/energy-performance-preference-epp.md) (CONCEPT)
- [EDC](/entities/edc.md) (CONCEPT)
- [nvoc](/entities/nvoc.md) (TOOL)
- [Zen 5](/entities/zen-5.md) (CONCEPT)
- [UCLK/MCLK](/entities/uclk-mclk.md) (CONCEPT)
- [TDC](/entities/tdc.md) (CONCEPT)
- [chnvml](/entities/chnvml.md) (TOOL)
- [Infinity Fabric Frequency (FCLK)](/entities/infinity-fabric-frequency-fclk.md) (CONCEPT)
- [Precision Boost Overdrive (PBO)](/entities/precision-boost-overdrive-pbo.md) (CONCEPT)
- [PPT](/entities/ppt.md) (CONCEPT)
- [GRUB](/entities/grub.md) (SYSTEM)
- [Integrated GPU (iGPU)](/entities/integrated-gpu-igpu.md) (HARDWARE)
- [NVMe](/entities/nvme.md) (TOOL)
- [MediaTek MT7925](/entities/mediatek-mt7925.md) (HARDWARE)
- [Flash Translation Layer (FTL)](/entities/flash-translation-layer-ftl.md) (CONCEPT)
- [NetworkManager](/entities/networkmanager.md) (SYSTEM)
- [iw](/entities/iw.md) (TOOL)
- [systemd](/entities/systemd.md) (SYSTEM)
- [Google Takeout](/entities/google-takeout.md) (TOOL)
- [Curve Shaper](/entities/curve-shaper.md) (CONCEPT)
- [Phison PS5031-E31T](/entities/phison-ps5031-e31t.md) (HARDWARE)

## Relations
- Ubuntu Extreme Hardware Tuning → RELATED_TO → Zen 5
- ASRock X870E Taichi Lite User Manual → RELATED_TO → Zen 5
- Zen 5 → PART_OF → AMI UEFI
- Precision Boost Overdrive (PBO) → PART_OF → AMI UEFI
- PPT → RELATED_TO → Precision Boost Overdrive (PBO)
- TDC → RELATED_TO → Precision Boost Overdrive (PBO)
- EDC → RELATED_TO → Precision Boost Overdrive (PBO)
- Curve Optimizer → RELATED_TO → Curve Shaper
- Ubuntu Extreme Hardware Tuning → USES → amd-pstate-epp driver
- Energy Performance Preference (EPP) → RELATED_TO → amd-pstate-epp driver
- NVIDIA GeForce RTX 5070 Ti → RELATED_TO → Wayland
- NVIDIA GeForce RTX 5070 Ti → USES → NVIDIA driver
- GSP firmware → RELATED_TO → NVIDIA driver
- nvoc → USES → systemd
- nvidia-persistenced → RELATED_TO → systemd
- PRIME → RELATED_TO → NVIDIA GeForce RTX 5070 Ti
- chnvml → RELATED_TO → NVML
- chnvml → USES → NVIDIA GeForce RTX 5070 Ti
- KIOXIA Exceria Plus G4 → RELATED_TO → Host Memory Buffer (HMB)
- Host Memory Buffer (HMB) → RELATED_TO → Flash Translation Layer (FTL)
- NVMe → USES → KIOXIA Exceria Plus G4
- NVMe → USES → Phison PS5031-E31T
- APST → RELATED_TO → NVMe
- GRUB → USES → nvme_core.default_ps_max_latency_us
- Realtek RTL8126 → USES → ethtool
- realtek-r8126-dkms → RELATED_TO → Realtek RTL8126
- MediaTek MT7925 → RELATED_TO → NetworkManager
- MediaTek MT7925 → USES → iw
- Google Takeout → RELATED_TO → Ubuntu Extreme Hardware Tuning
- Google Takeout → RELATED_TO → ASRock X870E Taichi Lite User Manual
- Ubuntu Extreme Hardware Tuning → USES → Linux scheduler
- NVIDIA GeForce RTX 5070 Ti → USES → chnvml
