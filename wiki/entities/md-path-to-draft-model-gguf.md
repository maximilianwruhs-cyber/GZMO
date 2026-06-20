---
type: entity
title: -md /path/to/draft-model.gguf
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# -md /path/to/draft-model.gguf

Type: TOOL

## From [optimizing-nvidia-blackwell-sm120-part1-micro02](/entities/optimizing-nvidia-blackwell-sm120-part1-micro02.md) (2026-06-09)
- Specifies the exact path to the aligned draft model.
- Speculative decoding leverages a smaller "draft" model to predict the next K tokens.
- Requires a smaller draft model of the identical architecture family.
- The small draft model evaluates quickly, generating a sequence of K tokens.
- Consumes some VRAM and execution threads.
- Applying a 0.5B draft model to a 27B parameter target architecture improved throughput.
