---
type: source
title: drive-research-automating-linux-hardware-detection-micro03
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# drive-research-automating-linux-hardware-detection-micro03

Ingested source summary (2026-06-09).

## Entities
- [[linux-capabilities|Linux Capabilities]] (CONCEPT)
- [[smbios|SMBIOS]] (CONCEPT)
- [[linux-kernel-driver-database-lkddb|Linux Kernel Driver DataBase (LKDDb)]] (SYSTEM)
- [[lazarus|Lazarus]] (ORGANIZATION)
- [[nvidia-drive-os-safety-framework|NVIDIA DRIVE OS safety framework]] (SYSTEM)
- [[kubernetes|Kubernetes]] (SYSTEM)
- [[ansible|Ansible]] (TOOL)
- [[cap-audit-read|CAP_AUDIT_READ]] (CONCEPT)
- [[payment-card-industry-data-security-standard-pci-dss|Payment Card Industry Data Security Standard (PCI DSS)]] (CONCEPT)
- [[devtmpfs|devtmpfs]] (SYSTEM)
- [[cpu-model-specific-registers-msrs|CPU Model-Specific Registers (MSRs)]] (CONCEPT)
- [[udev-rules|udev rules]] (CONCEPT)
- [[cap-sys-admin|CAP_SYS_ADMIN]] (CONCEPT)
- [[redfish-protocols|Redfish protocols]] (CONCEPT)
- [[dev-i2c-dev|/dev/i2c-dev]] (SYSTEM)
- [[cap-checkpoint-restore|CAP_CHECKPOINT_RESTORE]] (CONCEPT)
- [[hw-probe|hw-probe]] (TOOL)
- [[advanced-persistent-threats-apts|advanced persistent threats (APTs)]] (CONCEPT)
- [[sysfs-filesystem|sysfs filesystem]] (SYSTEM)
- [[cis-critical-controls|CIS Critical Controls]] (CONCEPT)
- [[hwinfo|hwinfo]] (TOOL)
- [[systemd-udevd|systemd-udevd]] (SYSTEM)
- [[userspace|userspace]] (CONCEPT)
- [[linux-hardware-org|linux-hardware.org]] (ORGANIZATION)
- [[cap-bpf|CAP_BPF]] (CONCEPT)
- [[github|GitHub]] (ORGANIZATION)
- [[cap-block-suspend|CAP_BLOCK_SUSPEND]] (CONCEPT)
- [[bmc|BMC]] (SYSTEM)
- [[redhat-rhel-mgmt|redhat.rhel_mgmt]] (ORGANIZATION)
- [[inter-integrated-circuit-i2c-protocol|Inter-Integrated Circuit (I2C) protocol]] (CONCEPT)
- [[sd-device|sd-device]] (CONCEPT)
- [[community-general-redfish-info|community.general.redfish_info]] (TOOL)
- [[dmidecode|dmidecode]] (TOOL)
- [[nist-cybersecurity-framework|NIST Cybersecurity Framework]] (CONCEPT)
- [[fancy-bear|Fancy Bear]] (ORGANIZATION)
- [[setcap|setcap]] (TOOL)
- [[lspci|lspci]] (TOOL)
- [[peripheral-component-interconnect-pci|Peripheral Component Interconnect (PCI)]] (CONCEPT)

## Relations
- community.general.redfish_info → PART_OF → redhat.rhel_mgmt
- community.general.redfish_info → PART_OF → Ansible
- Redfish protocols → RELATED_TO → BMC
- Redfish protocols → USES → SMBIOS
- Inter-Integrated Circuit (I2C) protocol → RELATED_TO → /dev/i2c-dev
- userspace → USES → Inter-Integrated Circuit (I2C) protocol
- hw-probe → USES → lspci
- hw-probe → USES → hwinfo
- hw-probe → USES → dmidecode
- hw-probe → RELATED_TO → Linux Kernel Driver DataBase (LKDDb)
- hw-probe → RELATED_TO → linux-hardware.org
- linux-hardware.org → RELATED_TO → Linux Kernel Driver DataBase (LKDDb)
- linux-hardware.org → RELATED_TO → GitHub
- Linux Capabilities → PART_OF → Linux Kernel Driver DataBase (LKDDb)
- CAP_SYS_ADMIN → PART_OF → Linux Capabilities
- CAP_AUDIT_READ → PART_OF → Linux Capabilities
- CAP_BPF → PART_OF → Linux Capabilities
- CAP_BLOCK_SUSPEND → PART_OF → Linux Capabilities
- CAP_CHECKPOINT_RESTORE → PART_OF → Linux Capabilities
- setcap → TOOL → Linux Capabilities
- Kubernetes → RELATED_TO → Linux Capabilities
- advanced persistent threats (APTs) → RELATED_TO → Fancy Bear
- advanced persistent threats (APTs) → RELATED_TO → Lazarus
- sysfs filesystem → PART_OF → Linux Kernel Driver DataBase (LKDDb)
- devtmpfs → PART_OF → Linux Kernel Driver DataBase (LKDDb)
- systemd-udevd → PART_OF → Linux Kernel Driver DataBase (LKDDb)
- sd-device → RELATED_TO → Ansible
- Ansible → RELATED_TO → Linux Kernel Driver DataBase (LKDDb)
