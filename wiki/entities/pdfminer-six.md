---
type: entity
title: PDFMiner.six
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# PDFMiner.six

Type: TOOL

## From [drive-research-pdf-text-vs-scan-detection-heuristics-micro03](/entities/drive-research-pdf-text-vs-scan-detection-heuristics-micro03.md) (2026-06-09)
- A pure-Python parser focused heavily on logical layout reconstruction.
- Does not perform any OCR functions; it strictly reads the existing PostScript commands to rebuild the document hierarchy.
- Offers a highly granular object classification system.
- Universally slower than PyMuPDF due to Python's execution overhead.
