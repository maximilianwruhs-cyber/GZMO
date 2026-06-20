---
type: entity
title: Layer-Splitting Mode
created: 2026-06-08
updated: 2026-06-08
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# Layer-Splitting Mode

Type: CONCEPT

## From [architectural-blueprints-for-sovereign-frankenmoe-part2](/entities/architectural-blueprints-for-sovereign-frankenmoe-part2.md) (2026-06-08)
- It is implemented using the `--split-mode layer` flag.
- It pins 100% of the model weights in VRAM across an asymmetric hardware pool.
- It slices the transformer architecture horizontally along layer boundaries.

## From [drive-research-so-what-is-your-final-model-constellation](/entities/drive-research-so-what-is-your-final-model-constellation.md) (2026-06-08)
- Slices the transformer architecture horizontally along layer boundaries.
- Pins 100% of model weights in VRAM.
- Transfers a small hidden state activation vector between layer steps.
