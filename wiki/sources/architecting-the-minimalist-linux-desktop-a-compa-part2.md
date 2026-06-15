---
type: source
title: architecting-the-minimalist-linux-desktop-a-compa-part2
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# architecting-the-minimalist-linux-desktop-a-compa-part2

Ingested source summary (2026-06-08).

## Entities
- [[master-boot-record-mbr|Master Boot Record (MBR)]] (CONCEPT)
- [[secure-boot|Secure Boot]] (CONCEPT)
- [[isohybrid-optical-overlays|isohybrid optical overlays]] (TOOL)
- [[ventoy|Ventoy]] (TOOL)
- [[microsoft-media-creation-tool|Microsoft Media Creation Tool]] (TOOL)
- [[rufus|Rufus]] (TOOL)
- [[mokmanager|MokManager]] (TOOL)
- [[unified-efi-forum|Unified EFI Forum]] (ORGANIZATION)
- [[wimtools|wimtools]] (TOOL)
- [[hybrid-mbrs|Hybrid MBRs]] (CONCEPT)
- [[yumi|YUMI]] (TOOL)
- [[woeusb|WoeUSB]] (TOOL)
- [[unified-extensible-firmware-interface-uefi|Unified Extensible Firmware Interface (UEFI)]] (SYSTEM)
- [[guid-partition-table-gpt|GUID Partition Table (GPT)]] (CONCEPT)
- [[fdisk|fdisk]] (TOOL)
- [[mbr2gpt|mbr2gpt]] (TOOL)
- [[legacy-basic-input-output-system-bios|Legacy Basic Input/Output System (BIOS)]] (SYSTEM)
- [[protective-mbr|Protective MBR]] (CONCEPT)
- [[intel-boot-initiative-ibi|Intel Boot Initiative (IBI)]] (ORGANIZATION)
- [[compatibility-support-module-csm|Compatibility Support Module (CSM)]] (CONCEPT)
- [[uefi-ntfs-drivers|UEFI:NTFS drivers]] (CONCEPT)
- [[csm-legacy-mode|CSM legacy mode]] (CONCEPT)
- [[uefi-forum|UEFI Forum]] (ORGANIZATION)
- [[hybrid-mbr|Hybrid MBR]] (CONCEPT)
- [[el-torito-boot-catalog-standard|El Torito boot catalog standard]] (CONCEPT)
- [[gdisk|gdisk]] (TOOL)
- [[fat32|FAT32]] (CONCEPT)
- [[microsoft-windows|Microsoft Windows]] (SYSTEM)
- [[grub2|GRUB2]] (SYSTEM)
- [[efi-system-partition-esp|EFI System Partition (ESP)]] (CONCEPT)
- [[winusb|WinUSB]] (TOOL)
- [[windows-11|Windows 11]] (SYSTEM)
- [[balenaetcher|BalenaEtcher]] (TOOL)
- [[deployment-image-servicing-and-management-dism|Deployment Image Servicing and Management (DISM)]] (TOOL)
- [[universal-disk-format-udf|Universal Disk Format (UDF)]] (CONCEPT)
- [[raspberry-pi-3|Raspberry Pi 3]] (SYSTEM)
- [[iso-9660|ISO 9660]] (CONCEPT)

## Relations
- Unified Extensible Firmware Interface (UEFI) → PART_OF → Intel Boot Initiative (IBI)
- Unified Extensible Firmware Interface (UEFI) → PART_OF → Unified EFI Forum
- Unified Extensible Firmware Interface (UEFI) → USES → GUID Partition Table (GPT)
- Legacy Basic Input/Output System (BIOS) → USES → Master Boot Record (MBR)
- CSM → RELATED_TO → Legacy Basic Input/Output System (BIOS)
- CSM → RELATED_TO → Unified Extensible Firmware Interface (UEFI)
- Secure Boot → PART_OF → Unified Extensible Firmware Interface (UEFI)
- Hybrid MBR → RELATED_TO → Master Boot Record (MBR)
- isohybrid optical overlays → USES → ISO 9660
- wimtools → RELATED_TO → DISM
- Rufus → USES → UEFI:NTFS drivers
- Ventoy → USES → GRUB2
- Ventoy → USES → MokManager
- Microsoft Media Creation Tool → USES → FAT32
- BalenaEtcher → RELATED_TO → isohybrid optical overlays
- WoeUSB → USES → NTFS
- WoeUSB → USES → Master Boot Record (MBR)
- WoeUSB → USES → Unified Extensible Firmware Interface (UEFI)
- YUMI → USES → NTFS
- YUMI → USES → FAT32
- WinUSB → USES → NTFS
- WinUSB → USES → FAT32
- GRUB2 → USES → GUID Partition Table (GPT)
- Unified Extensible Firmware Interface (UEFI) → PART_OF → UEFI Forum
- Protective MBR → RELATED_TO → GUID Partition Table (GPT)
- fdisk → USES → GUID Partition Table (GPT)
- gdisk → USES → GUID Partition Table (GPT)
- Raspberry Pi 3 → USES → Master Boot Record (MBR)
- Raspberry Pi 3 → USES → Hybrid MBRs
- Microsoft Windows → USES → Unified Extensible Firmware Interface (UEFI)
- Microsoft Windows → RELATED_TO → Master Boot Record (MBR)
- mbr2gpt → USES → Microsoft Windows
- Intel Boot Initiative (IBI) → USES → Unified Extensible Firmware Interface (UEFI)
- Unified Extensible Firmware Interface (UEFI) → RELATED_TO → GUID Partition Table (GPT)
- Windows 11 → USES → Unified Extensible Firmware Interface (UEFI)
- Windows 11 → USES → Secure Boot
- CSM legacy mode → PART_OF → Unified Extensible Firmware Interface (UEFI)
