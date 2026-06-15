---
type: entity
title: Hugging Face Transformers Integration
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Hugging Face Transformers Integration

Type: CONCEPT

## From [[drive-research-speicherbandbreiten-engpass-memory-wall|drive-research-speicherbandbreiten-engpass-memory-wall]] (2026-06-08)
- Integrates TurboQuant via the turboquant Python package.
- Speculative drafting is enabled via the assistant_model parameter in the .generate() API.
- Executes a 'hybrid decode' that saves VRAM but must dequantize historical vectors.
- A framework where TurboQuant and speculative decoding can be implemented.
- The turboquant Python package provides a TurboQuantCache class to replace standard caches.
- The turboquant Python package provides a TurboQuantCache class.
- This integration executes a 'hybrid decode'.
