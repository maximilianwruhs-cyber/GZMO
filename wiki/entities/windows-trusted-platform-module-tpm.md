---
type: entity
title: Windows Trusted Platform Module (TPM)
created: 2026-06-08
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# Windows Trusted Platform Module (TPM)

Type: SYSTEM

## From [architectural-strategy-for-stealthy-portable-cli-a](/entities/architectural-strategy-for-stealthy-portable-cli-a.md) (2026-06-08)
- Relying on host-specific hardware cryptographic mechanisms—such as the Windows Trusted Platform Module (TPM)—violates the strict cross-platform portability requirement.

## From [drive-research-architecting-zero-configuration-portable-agents-s-micro03](/entities/drive-research-architecting-zero-configuration-portable-agents-s-micro03.md) (2026-06-09)
- A host-specific hardware cryptographic mechanism.
- Violates cross-platform portability requirement if relied upon.
- Host-specific hardware cryptographic mechanism.
- Violates cross-platform portability if relied upon.
- Windows Trusted Platform Module.
- Hardware discovery interrogates RegGetValue for HKLM NVIDIA keys and attempts dynamic loading via LoadLibrary for nvcuda.dll.
- Captive portal detection uses http://www.msftconnecttest.com/connecttest.txt.
- Uses Schannel on Windows for native OS TLS wrappers.
- Uses GetExtendedTcpTable API from the IP Helper (Iphlpapi) interface.
- Uses VirtualLock API for memory locking.
