---
type: entity
title: Medusa
created: 2026-06-08
updated: 2026-06-09
sources: 6
tags: []
status: draft
gzmo_synthetic: true
---






# Medusa

Type: SYSTEM

## From [obolus-vs-codium-extension-konzept-research-part2](/entities/obolus-vs-codium-extension-konzept-research-part2.md) (2026-06-08)
- architecture
- Instead of a separate model, researchers surgically attach a microscopic "prediction head" (a 1-layer neural network) to the very end of a standard large model.

## From [drive-research-erbandbreite-und-latenzengpässe](/entities/drive-research-erbandbreite-und-latenzengp-sse.md) (2026-06-08)
- An architecture that augments the target model with multiple parallel decoding heads.
- Generates a comprehensive tree of multiple candidate continuations simultaneously.
- Operates in a non-autoregressive manner.

## From [drive-research-speicherbandbreiten-engpass-memory-wall](/entities/drive-research-speicherbandbreiten-engpass-memory-wall.md) (2026-06-08)
- An advanced drafting architecture enabled by TurboQuant.
- Adds multiple extra language model heads to predict tokens.
- A non-autoregressive Medusa head is less efficient than autoregressive EAGLE heads.
- A self-speculating architecture.
- Operates at the hidden states level.
- Adds multiple extra language model heads.

## From [the-architecture-of-speculative-decoding-and-infer-part1-micro05](/entities/the-architecture-of-speculative-decoding-and-infer-part1-micro05.md) (2026-06-09)
- An example of a multi-branch drafting head enabled by TurboQuant.

## From [the-architecture-of-speculative-decoding-and-infer-part1-micro06](/entities/the-architecture-of-speculative-decoding-and-infer-part1-micro06.md) (2026-06-09)
- Does not rely on an external language model for drafting
- Operates at the hidden states level
- Enhances the base model by adding multiple extra language model heads
- The first head predicts the immediate next token (t+1)
- The second head predicts the token after that (t+2)
- Utilizes a TopK approach to create multiple potential paths

## From [the-architecture-of-speculative-decoding-and-infer-part2-micro02](/entities/the-architecture-of-speculative-decoding-and-infer-part2-micro02.md) (2026-06-09)
- Augments the target model by appending multiple parallel decoding heads atop the final hidden state.
- Generates a comprehensive tree of multiple candidate continuations simultaneously.
- Operates in a non-autoregressive manner.
