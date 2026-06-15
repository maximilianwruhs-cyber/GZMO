---
type: entity
title: Windows Registry
created: 2026-06-08
updated: 2026-06-09
sources: 3
tags: []
status: draft
gzmo_synthetic: true
---



# Windows Registry

Type: SYSTEM

## From [[architectural-strategy-for-stealthy-portable-cli-a|architectural-strategy-for-stealthy-portable-cli-a]] (2026-06-08)
- Used on Windows architectures for passive interrogation of NVIDIA CUDA ecosystem.
- Querying HKEY_LOCAL_MACHINE\SOFTWARE\NVIDIA Corporation\Global indicates NVIDIA driver installation.
- Querying is a low-privileged operation, invisible as a malicious indicator.

## From [[gzmo-soul-merged-new-part1|gzmo-soul-merged-new-part1]] (2026-06-09)
- Windows registration database where programs leave complex traces.
- Key paths for auditing include HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall* and HKCU for user-space software.

## From [[drive-research-architektur-und-implementierung-intelligenter-ki-a-micro01|drive-research-architektur-und-implementierung-intelligenter-ki-a-micro01]] (2026-06-09)
- Stores metadata of installed programs in Uninstall branches.
- Contains paths like HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\* and Wow6432Node equivalents.
- Also includes HKCU for user-space software.
