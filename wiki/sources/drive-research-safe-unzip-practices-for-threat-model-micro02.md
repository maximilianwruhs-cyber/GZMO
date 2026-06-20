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
- [canonical path validation](/entities/canonical-path-validation.md) (CONCEPT)
- [Jenkins](/entities/jenkins.md) (PROJECT)
- [42.zip](/entities/42-zip.md) (CONCEPT)
- [symbolic links](/entities/symbolic-links.md) (CONCEPT)
- [Trend Micro (WFBS)](/entities/trend-micro-wfbs.md) (TOOL)
- [CVE-2025-11001](/entities/cve-2025-11001.md) (CONCEPT)
- [Google Workspace (Gmail)](/entities/google-workspace-gmail.md) (SYSTEM)
- [fork bombs](/entities/fork-bombs.md) (CONCEPT)
- [OWASP Guidelines](/entities/owasp-guidelines.md) (CONCEPT)
- [path traversal attacks](/entities/path-traversal-attacks.md) (CONCEPT)
- [HP Fortify](/entities/hp-fortify.md) (PROJECT)
- [SonarQube](/entities/sonarqube.md) (PROJECT)
- [Apache Hadoop](/entities/apache-hadoop.md) (PROJECT)
- [Control Groups (cgroups)](/entities/control-groups-cgroups.md) (SYSTEM)
- [Symantec Protection Engine](/entities/symantec-protection-engine.md) (TOOL)
- [GitHub Actions (CI/CD)](/entities/github-actions-ci-cd.md) (SYSTEM)
- [Microsoft Office Data Model](/entities/microsoft-office-data-model.md) (CONCEPT)
- [os.path.abspath](/entities/os-path-abspath.md) (TOOL)
- [Nginx Web Server](/entities/nginx-web-server.md) (SYSTEM)
- [David Fifield](/entities/david-fifield.md) (PERSON)
- [filepath.Join](/entities/filepath-join.md) (TOOL)
- [Microsoft Office 365 / Exchange](/entities/microsoft-office-365-exchange.md) (SYSTEM)
- [7-Zip](/entities/7-zip.md) (TOOL)
- [ClamAV Engine](/entities/clamav-engine.md) (TOOL)
- [strings.HasPrefix](/entities/strings-hasprefix.md) (TOOL)
- [Zip Slip](/entities/zip-slip.md) (CONCEPT)
- [process exhaustion attacks](/entities/process-exhaustion-attacks.md) (CONCEPT)
- [os.path.normpath](/entities/os-path-normpath.md) (TOOL)
- [Snyk Code Security](/entities/snyk-code-security.md) (TOOL)
- [ulimit](/entities/ulimit.md) (TOOL)
- [systemd](/entities/systemd.md) (SYSTEM)
- [Zip64 Extension](/entities/zip64-extension.md) (CONCEPT)

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
