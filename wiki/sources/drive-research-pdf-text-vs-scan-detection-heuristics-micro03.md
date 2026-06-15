---
type: source
title: drive-research-pdf-text-vs-scan-detection-heuristics-micro03
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# drive-research-pdf-text-vs-scan-detection-heuristics-micro03

Ingested source summary (2026-06-09).

## Entities
- [[ocrmypdf|OCRmyPDF]] (TOOL)
- [[poppler-utilities|Poppler Utilities]] (TOOL)
- [[pymupdf|PyMuPDF]] (TOOL)
- [[tesseract|Tesseract]] (TOOL)
- [[paddleocr|PaddleOCR]] (TOOL)
- [[pdfminer-six|PDFMiner.six]] (TOOL)
- [[mupdf|MuPDF]] (SYSTEM)
- [[dropzone-convert-module|dropzone_convert module]] (SYSTEM)
- [[vectorized-fonts|Vectorized Fonts]] (CONCEPT)
- [[text-layer-obfuscation|Text Layer Obfuscation]] (CONCEPT)
- [[hybrid-pipeline|Hybrid Pipeline]] (CONCEPT)
- [[language-models|Language Models]] (SYSTEM)
- [[ocr-sandwich|OCR Sandwich]] (CONCEPT)

## Relations
- PyMuPDF → PART_OF → MuPDF
- PyMuPDF → RELATED_TO → Text Layer Obfuscation
- PyMuPDF → RELATED_TO → OCR Sandwich
- PDFMiner.six → RELATED_TO → Text Layer Obfuscation
- Poppler Utilities → RELATED_TO → Text Layer Obfuscation
- OCRmyPDF → USES → PyMuPDF
- OCRmyPDF → RELATED_TO → Text Layer Obfuscation
- dropzone_convert module → USES → Poppler Utilities
- dropzone_convert module → USES → PyMuPDF
- dropzone_convert module → RELATED_TO → Hybrid Pipeline
- Hybrid Pipeline → RELATED_TO → Language Models
