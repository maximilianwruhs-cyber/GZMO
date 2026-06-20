---
type: entity
title: canonical path validation
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# canonical path validation

Type: CONCEPT

## From [drive-research-safe-unzip-practices-for-threat-model-micro02](/entities/drive-research-safe-unzip-practices-for-threat-model-micro02.md) (2026-06-09)
- Universally recognized and required mitigation for Zip Slip.
- Involves computing the canonical, absolute path of the destination directory and the file to be created.
- Requires verifying that the canonical path of the extracted file begins with the canonical path of the destination directory.
