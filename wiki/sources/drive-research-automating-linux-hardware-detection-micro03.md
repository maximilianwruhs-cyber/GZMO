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
- [Linux Capabilities](/entities/linux-capabilities.md) (CONCEPT)
- [SMBIOS](/entities/smbios.md) (CONCEPT)
- [Linux Kernel Driver DataBase (LKDDb)](/entities/linux-kernel-driver-database-lkddb.md) (SYSTEM)
- [Lazarus](/entities/lazarus.md) (ORGANIZATION)
- [NVIDIA DRIVE OS safety framework](/entities/nvidia-drive-os-safety-framework.md) (SYSTEM)
- [Kubernetes](/entities/kubernetes.md) (SYSTEM)
- [Ansible](/entities/ansible.md) (TOOL)
- [CAP_AUDIT_READ](/entities/cap-audit-read.md) (CONCEPT)
- [Payment Card Industry Data Security Standard (PCI DSS)](/entities/payment-card-industry-data-security-standard-pci-dss.md) (CONCEPT)
- [devtmpfs](/entities/devtmpfs.md) (SYSTEM)
- [CPU Model-Specific Registers (MSRs)](/entities/cpu-model-specific-registers-msrs.md) (CONCEPT)
- [udev rules](/entities/udev-rules.md) (CONCEPT)
- [CAP_SYS_ADMIN](/entities/cap-sys-admin.md) (CONCEPT)
- [Redfish protocols](/entities/redfish-protocols.md) (CONCEPT)
- [/dev/i2c-dev](/entities/dev-i2c-dev.md) (SYSTEM)
- [CAP_CHECKPOINT_RESTORE](/entities/cap-checkpoint-restore.md) (CONCEPT)
- [hw-probe](/entities/hw-probe.md) (TOOL)
- [advanced persistent threats (APTs)](/entities/advanced-persistent-threats-apts.md) (CONCEPT)
- [sysfs filesystem](/entities/sysfs-filesystem.md) (SYSTEM)
- [CIS Critical Controls](/entities/cis-critical-controls.md) (CONCEPT)
- [hwinfo](/entities/hwinfo.md) (TOOL)
- [systemd-udevd](/entities/systemd-udevd.md) (SYSTEM)
- [userspace](/entities/userspace.md) (CONCEPT)
- [linux-hardware.org](/entities/linux-hardware-org.md) (ORGANIZATION)
- [CAP_BPF](/entities/cap-bpf.md) (CONCEPT)
- [GitHub](/entities/github.md) (ORGANIZATION)
- [CAP_BLOCK_SUSPEND](/entities/cap-block-suspend.md) (CONCEPT)
- [BMC](/entities/bmc.md) (SYSTEM)
- [redhat.rhel_mgmt](/entities/redhat-rhel-mgmt.md) (ORGANIZATION)
- [Inter-Integrated Circuit (I2C) protocol](/entities/inter-integrated-circuit-i2c-protocol.md) (CONCEPT)
- [sd-device](/entities/sd-device.md) (CONCEPT)
- [community.general.redfish_info](/entities/community-general-redfish-info.md) (TOOL)
- [dmidecode](/entities/dmidecode.md) (TOOL)
- [NIST Cybersecurity Framework](/entities/nist-cybersecurity-framework.md) (CONCEPT)
- [Fancy Bear](/entities/fancy-bear.md) (ORGANIZATION)
- [setcap](/entities/setcap.md) (TOOL)
- [lspci](/entities/lspci.md) (TOOL)
- [Peripheral Component Interconnect (PCI)](/entities/peripheral-component-interconnect-pci.md) (CONCEPT)

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
