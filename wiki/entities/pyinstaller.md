---
type: entity
title: PyInstaller
created: 2026-06-08
updated: 2026-06-10
sources: 3
tags: []
status: draft
gzmo_synthetic: true
---



# PyInstaller

Type: TOOL

## From [architectural-strategy-for-stealthy-portable-cli-a](/entities/architectural-strategy-for-stealthy-portable-cli-a.md) (2026-06-08)
- Packaging utility for Python.
- Does not natively compile Python scripts into raw machine code.
- Packages interpreter, bytecode, and shared libraries into a compressed archive appended to a C bootloader.
- PyInstaller packages a Python interpreter and scripts into a bootloader.
- Upon execution, it extracts these files into the host's temporary directory (e.g., %TEMP%_MEIxxxx or /tmp).
- EDR platforms aggressively monitor execution originating from temporary directories.

## From [drive-research-to-product-engineering-leadership](/entities/drive-research-to-product-engineering-leadership.md) (2026-06-08)
- Used with Python.
- Packages Python interpreter and scripts.
- Extracts files into temporary directories, triggering EDR.

## From [drive-research-architecting-zero-configuration-portable-agents-s-micro02](/entities/drive-research-architecting-zero-configuration-portable-agents-s-micro02.md) (2026-06-10)
- Packages Python interpreter, bytecode, and shared libraries into a compressed archive.
- Uses a compiled C bootloader for execution.
- Triggers Sysmon Event ID 11 and Event ID 1 due to file I/O and process creation.
