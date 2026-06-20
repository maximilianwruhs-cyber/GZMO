---
type: source
title: drive-research-safe-unzip-practices-for-threat-model-micro03
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# drive-research-safe-unzip-practices-for-threat-model-micro03

Ingested source summary (2026-06-09).

## Entities
- [single-user threat model](/entities/single-user-threat-model.md) (CONCEPT)
- [OOXML documents](/entities/ooxml-documents.md) (CONCEPT)
- [ZIP (file format)](/entities/zip-file-format.md) (CONCEPT)
- [Node's path.resolve()](/entities/node-s-path-resolve.md) (TOOL)
- [Zip Slip](/entities/zip-slip.md) (CONCEPT)
- [Java's getCanonicalPath()](/entities/java-s-getcanonicalpath.md) (TOOL)
- [XML entity expansion](/entities/xml-entity-expansion.md) (CONCEPT)
- [ulimit -S -u 400](/entities/ulimit-s-u-400.md) (TOOL)
- [Symlink Resolution](/entities/symlink-resolution.md) (CONCEPT)
- [Process Boundary Enforcement](/entities/process-boundary-enforcement.md) (CONCEPT)
- [Strict Canonical Path Validation](/entities/strict-canonical-path-validation.md) (CONCEPT)
- [Python's os.path.normpath()](/entities/python-s-os-path-normpath.md) (TOOL)
- [Compression bombs](/entities/compression-bombs.md) (CONCEPT)
- [Fork Bomb](/entities/fork-bomb.md) (CONCEPT)
- [systemd](/entities/systemd.md) (SYSTEM)
- [TasksMax directives](/entities/tasksmax-directives.md) (TOOL)

## Relations
- Symlink Resolution → RELATED_TO → single-user threat model
- Strict Canonical Path Validation → RELATED_TO → Zip Slip
- Strict Canonical Path Validation → USES → Java's getCanonicalPath()
- Strict Canonical Path Validation → USES → Node's path.resolve()
- Strict Canonical Path Validation → USES → Python's os.path.normpath()
- Process Boundary Enforcement → RELATED_TO → Fork Bomb
- Process Boundary Enforcement → USES → ulimit -S -u 400
- Process Boundary Enforcement → USES → systemd
- Process Boundary Enforcement → USES → TasksMax directives
- XML entity expansion → PART_OF → OOXML documents
- Compression bombs → RELATED_TO → ZIP (file format)
- Zip Slip → RELATED_TO → ZIP (file format)
- single-user threat model → RELATED_TO → Symlink Resolution
