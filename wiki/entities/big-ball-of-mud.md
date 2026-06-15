---
type: entity
title: Big Ball of Mud
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Big Ball of Mud

Type: CONCEPT

## From [[google-antigravity-the-architects-configuration-micro06|google-antigravity-the-architects-configuration-micro06]] (2026-06-09)
- Ein Legacy Anti-Pattern.
- Module nutzen interne Details anderer Module oder rufen sich in geschlossener Schleife auf.
- Bricht den Dependency Graph, verursacht Build-Fehler und macht isoliertes Testen unmöglich.
