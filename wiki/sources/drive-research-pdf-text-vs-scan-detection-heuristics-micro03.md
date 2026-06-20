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
- [OCRmyPDF](/entities/ocrmypdf.md) (TOOL)
- [Poppler Utilities](/entities/poppler-utilities.md) (TOOL)
- [PyMuPDF](/entities/pymupdf.md) (TOOL)
- [Tesseract](/entities/tesseract.md) (TOOL)
- [PaddleOCR](/entities/paddleocr.md) (TOOL)
- [PDFMiner.six](/entities/pdfminer-six.md) (TOOL)
- [MuPDF](/entities/mupdf.md) (SYSTEM)
- [dropzone_convert module](/entities/dropzone-convert-module.md) (SYSTEM)
- [Vectorized Fonts](/entities/vectorized-fonts.md) (CONCEPT)
- [Text Layer Obfuscation](/entities/text-layer-obfuscation.md) (CONCEPT)
- [Hybrid Pipeline](/entities/hybrid-pipeline.md) (CONCEPT)
- [Language Models](/entities/language-models.md) (SYSTEM)
- [OCR Sandwich](/entities/ocr-sandwich.md) (CONCEPT)

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
