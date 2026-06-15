---
type: entity
title: macOS
created: 2026-06-08
updated: 2026-06-10
sources: 9
tags: []
status: draft
gzmo_synthetic: true
---









# macOS

Type: SYSTEM

## From [[architectural-strategy-for-stealthy-portable-cli-a|architectural-strategy-for-stealthy-portable-cli-a]] (2026-06-08)
- One of the architectures for which cross-platform functionality is required.
- Uses /tmp/_MEIxxxxxx for temporary directory extraction by PyInstaller.
- Features systemic sandboxing, Gatekeeper, XProtect, and Endpoint Security Framework.

## From [[drive-research-hermes-session-storage-migration-analysis|drive-research-hermes-session-storage-migration-analysis]] (2026-06-08)
- Hermes has extensive computer-use capabilities primarily for macOS environments.
- Requires macOS Accessibility & Screen Recording permissions for visual workflow automation.

## From [[drive-research-llamacpp-gpu-memory-reporting-bug|drive-research-llamacpp-gpu-memory-reporting-bug]] (2026-06-08)
- Follows the LP64 data model.
- Discrete AMD GPUs run on macOS via the Metal backend.

## From [[drive-research-advanced-typescript-execution-in-the-bun-runtime-micro03|drive-research-advanced-typescript-execution-in-the-bun-runtime-micro03]] (2026-06-09)
- It achieves this by heavily utilizing OS-specific low-level tricks, such as clonefile on macOS and hardlinks on Linux, to copy files instantaneously from a global binary cache.

## From [[drive-research-architecting-zero-configuration-portable-agents-s-micro03|drive-research-architecting-zero-configuration-portable-agents-s-micro03]] (2026-06-09)
- Hardware discovery routes to executing sysctlbyname for hw.optional.mps.
- Captive portal detection uses http://captive.apple.com/hotspot-detect.html.
- Uses Secure Transport on macOS for native OS TLS wrappers.
- Uses sysctl interface (net.inet.tcp.pcblist MIB) for socket table reconstruction.
- Uses mlock API for memory locking.

## From [[drive-research-bun-typescript-performance-tips-micro03|drive-research-bun-typescript-performance-tips-micro03]] (2026-06-09)
- Bun install uses clonefile on macOS.

## From [[drive-research-llm-inference-engine-audit-2026-micro02|drive-research-llm-inference-engine-audit-2026-micro02]] (2026-06-09)
- MLX seamlessly handles unified memory allocation on it without requiring developers to manually configure complex parameters.

## From [[drive-research-architecting-zero-configuration-portable-agents-s-micro02|drive-research-architecting-zero-configuration-portable-agents-s-micro02]] (2026-06-10)
- Target architecture with heavy sandboxing and Gatekeeper constraints.
- Uses sysctl and Endpoint Security Framework.

## From [[optimizing-nvidia-blackwell-sm120-part3-micro03|optimizing-nvidia-blackwell-sm120-part3-micro03]] (2026-06-10)
- Follows the LP64 data model.
