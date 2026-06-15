---
type: entity
title: PowerShell
created: 2026-06-09
updated: 2026-06-09
sources: 4
tags: []
status: draft
gzmo_synthetic: true
---





# PowerShell

Type: TOOL

## From [[gzmo-soul-merged-new-part1|gzmo-soul-merged-new-part1]] (2026-06-09)
- Used by the AI agent to generate and execute specialized scripts for auditing installation integrity on Windows.
- Cmdlets like Get-ItemProperty and Select-Object are used to extract parameters from the Windows Registry.
- Wird für native Registry-Zugriffe verwendet

## From [[drive-research-architektur-und-implementierung-intelligenter-ki-a-micro01|drive-research-architektur-und-implementierung-intelligenter-ki-a-micro01]] (2026-06-09)
- Used to generate and execute specialized scripts for auditing installation integrity.
- Utilizes Cmdlets like Get-ItemProperty and Select-Object.
- Can generate scripts to clean up orphaned registry entries.

## From [[drive-research-architektur-und-implementierung-intelligenter-ki-a-micro02|drive-research-architektur-und-implementierung-intelligenter-ki-a-micro02]] (2026-06-09)
- Can be used to read environment variables like 'Path' using [Environment]::GetEnvironmentVariable('Path', 'Machine').
- Can be used to write environment variables back to the registry using [Environment]::SetEnvironmentVariable('Path', $newPath, Target).
- Can be used to check path existence with Test-Path cmdlet.

## From [[gzmo-soul-merged-new-part2-micro05|gzmo-soul-merged-new-part2-micro05]] (2026-06-09)
- Used by agents to identify 'orphaned' registry entries and dead links.
- Used to correct the PATH variable.
