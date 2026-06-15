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
- [[linux-scheduler|Linux scheduler]] (SYSTEM)
- [[gsp-firmware|GSP firmware]] (CONCEPT)
- [[host-memory-buffer-hmb|Host Memory Buffer (HMB)]] (CONCEPT)
- [[bbr|BBR]] (CONCEPT)
- [[nvidia-driver|NVIDIA driver]] (TOOL)
- [[prime|PRIME]] (CONCEPT)
- [[realtek-r8126-dkms|realtek-r8126-dkms]] (TOOL)
- [[nvidia-geforce-rtx-5070-ti|NVIDIA GeForce RTX 5070 Ti]] (HARDWARE)
- [[curve-optimizer|Curve Optimizer]] (CONCEPT)
- [[ami-uefi|AMI UEFI]] (SYSTEM)
- [[apst|APST]] (CONCEPT)
- [[amd-pstate-epp-driver|amd-pstate-epp driver]] (SYSTEM)
- [[nvml|NVML]] (TOOL)
- [[wayland|Wayland]] (SYSTEM)
- [[realtek-rtl8126|Realtek RTL8126]] (HARDWARE)
- [[asrock-x870e-taichi-lite-user-manual|ASRock X870E Taichi Lite User Manual]] (BOOK)
- [[nvidia-persistenced|nvidia-persistenced]] (SYSTEM)
- [[ubuntu-extreme-hardware-tuning|Ubuntu Extreme Hardware Tuning]] (SYSTEM)
- [[ethtool|ethtool]] (TOOL)
- [[sysctl|sysctl]] (TOOL)
- [[kioxia-exceria-plus-g4|KIOXIA Exceria Plus G4]] (HARDWARE)
- [[energy-performance-preference-epp|Energy Performance Preference (EPP)]] (CONCEPT)
- [[edc|EDC]] (CONCEPT)
- [[nvoc|nvoc]] (TOOL)
- [[zen-5|Zen 5]] (CONCEPT)
- [[uclk-mclk|UCLK/MCLK]] (CONCEPT)
- [[tdc|TDC]] (CONCEPT)
- [[chnvml|chnvml]] (TOOL)
- [[infinity-fabric-frequency-fclk|Infinity Fabric Frequency (FCLK)]] (CONCEPT)
- [[precision-boost-overdrive-pbo|Precision Boost Overdrive (PBO)]] (CONCEPT)
- [[ppt|PPT]] (CONCEPT)
- [[grub|GRUB]] (SYSTEM)
- [[integrated-gpu-igpu|Integrated GPU (iGPU)]] (HARDWARE)
- [[nvme|NVMe]] (TOOL)
- [[mediatek-mt7925|MediaTek MT7925]] (HARDWARE)
- [[flash-translation-layer-ftl|Flash Translation Layer (FTL)]] (CONCEPT)
- [[networkmanager|NetworkManager]] (SYSTEM)
- [[iw|iw]] (TOOL)
- [[systemd|systemd]] (SYSTEM)
- [[google-takeout|Google Takeout]] (TOOL)
- [[curve-shaper|Curve Shaper]] (CONCEPT)
- [[phison-ps5031-e31t|Phison PS5031-E31T]] (HARDWARE)

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
