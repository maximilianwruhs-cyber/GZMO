---
type: entity
title: OCRmyPDF
created: 2026-06-09
updated: 2026-06-10
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# OCRmyPDF

Type: TOOL

## From [[drive-research-pdf-text-vs-scan-detection-heuristics-micro03|drive-research-pdf-text-vs-scan-detection-heuristics-micro03]] (2026-06-09)
- A highly popular utility for the eventual heavy OCR step.
- Includes built-in command-line arguments specifically engineered to prevent redundant processing.
- Relies internally on PyMuPDF (when installed with the [fitz] extra package) to intelligently evaluate whether a page requires OCR on a strict, per-page basis.

## From [[drive-research-pdf-text-vs-scan-detection-heuristics-micro04|drive-research-pdf-text-vs-scan-detection-heuristics-micro04]] (2026-06-10)
- Adds an OCR text layer to PDF files.
