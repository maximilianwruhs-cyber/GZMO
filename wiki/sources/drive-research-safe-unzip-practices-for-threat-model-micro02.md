---
type: source
title: drive-research-safe-unzip-practices-for-threat-model-micro02
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# drive-research-safe-unzip-practices-for-threat-model-micro02

Ingested source summary (2026-06-09).

## Entities
- [[canonical-path-validation|canonical path validation]] (CONCEPT)
- [[jenkins|Jenkins]] (PROJECT)
- [[42-zip|42.zip]] (CONCEPT)
- [[symbolic-links|symbolic links]] (CONCEPT)
- [[trend-micro-wfbs|Trend Micro (WFBS)]] (TOOL)
- [[cve-2025-11001|CVE-2025-11001]] (CONCEPT)
- [[google-workspace-gmail|Google Workspace (Gmail)]] (SYSTEM)
- [[fork-bombs|fork bombs]] (CONCEPT)
- [[owasp-guidelines|OWASP Guidelines]] (CONCEPT)
- [[path-traversal-attacks|path traversal attacks]] (CONCEPT)
- [[hp-fortify|HP Fortify]] (PROJECT)
- [[sonarqube|SonarQube]] (PROJECT)
- [[apache-hadoop|Apache Hadoop]] (PROJECT)
- [[control-groups-cgroups|Control Groups (cgroups)]] (SYSTEM)
- [[symantec-protection-engine|Symantec Protection Engine]] (TOOL)
- [[github-actions-ci-cd|GitHub Actions (CI/CD)]] (SYSTEM)
- [[microsoft-office-data-model|Microsoft Office Data Model]] (CONCEPT)
- [[os-path-abspath|os.path.abspath]] (TOOL)
- [[nginx-web-server|Nginx Web Server]] (SYSTEM)
- [[david-fifield|David Fifield]] (PERSON)
- [[filepath-join|filepath.Join]] (TOOL)
- [[microsoft-office-365-exchange|Microsoft Office 365 / Exchange]] (SYSTEM)
- [[7-zip|7-Zip]] (TOOL)
- [[clamav-engine|ClamAV Engine]] (TOOL)
- [[strings-hasprefix|strings.HasPrefix]] (TOOL)
- [[zip-slip|Zip Slip]] (CONCEPT)
- [[process-exhaustion-attacks|process exhaustion attacks]] (CONCEPT)
- [[os-path-normpath|os.path.normpath]] (TOOL)
- [[snyk-code-security|Snyk Code Security]] (TOOL)
- [[ulimit|ulimit]] (TOOL)
- [[systemd|systemd]] (SYSTEM)
- [[zip64-extension|Zip64 Extension]] (CONCEPT)

## Relations
- Zip Slip → RELATED_TO → canonical path validation
- Zip Slip → PART_OF → Apache Hadoop
- Zip Slip → PART_OF → Jenkins
- Zip Slip → PART_OF → HP Fortify
- Zip Slip → PART_OF → SonarQube
- canonical path validation → USES → os.path.normpath
- canonical path validation → USES → os.path.abspath
- canonical path validation → USES → filepath.Join
- canonical path validation → USES → strings.HasPrefix
- path traversal attacks → RELATED_TO → symbolic links
- CVE-2025-11001 → RELATED_TO → 7-Zip
- fork bombs → RELATED_TO → process exhaustion attacks
- process exhaustion attacks → USES → systemd
- process exhaustion attacks → USES → ulimit
- 7-Zip → RELATED_TO → Zip64 Extension
- 42.zip → RELATED_TO → David Fifield
