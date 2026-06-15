---
type: entity
title: Strict Canonical Path Validation
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Strict Canonical Path Validation

Type: CONCEPT

## From [[drive-research-safe-unzip-practices-for-threat-model-micro03|drive-research-safe-unzip-practices-for-threat-model-micro03]] (2026-06-09)
- Mandatory to prevent directory traversal exploits.
- Requires functions equivalent to Java's getCanonicalPath().
- Requires functions equivalent to Node's path.resolve().
- Requires functions equivalent to Python's os.path.normpath().
