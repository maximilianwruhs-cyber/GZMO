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
- [[single-user-threat-model|single-user threat model]] (CONCEPT)
- [[ooxml-documents|OOXML documents]] (CONCEPT)
- [[zip-file-format|ZIP (file format)]] (CONCEPT)
- [[node-s-path-resolve|Node's path.resolve()]] (TOOL)
- [[zip-slip|Zip Slip]] (CONCEPT)
- [[java-s-getcanonicalpath|Java's getCanonicalPath()]] (TOOL)
- [[xml-entity-expansion|XML entity expansion]] (CONCEPT)
- [[ulimit-s-u-400|ulimit -S -u 400]] (TOOL)
- [[symlink-resolution|Symlink Resolution]] (CONCEPT)
- [[process-boundary-enforcement|Process Boundary Enforcement]] (CONCEPT)
- [[strict-canonical-path-validation|Strict Canonical Path Validation]] (CONCEPT)
- [[python-s-os-path-normpath|Python's os.path.normpath()]] (TOOL)
- [[compression-bombs|Compression bombs]] (CONCEPT)
- [[fork-bomb|Fork Bomb]] (CONCEPT)
- [[systemd|systemd]] (SYSTEM)
- [[tasksmax-directives|TasksMax directives]] (TOOL)

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
