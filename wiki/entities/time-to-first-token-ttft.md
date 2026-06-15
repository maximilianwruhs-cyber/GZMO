---
type: entity
title: Time To First Token (TTFT)
created: 2026-06-09
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# Time To First Token (TTFT)

Type: CONCEPT

## From [[drive-research-32gb-vram-ai-reasoning-models-micro01|drive-research-32gb-vram-ai-reasoning-models-micro01]] (2026-06-09)
- Historically measured in milliseconds for predictive models
- In the reasoning paradigm, it includes latent reasoning durations
- Can extend to several seconds or tens of seconds for reasoning models

## From [[drive-research-llamacpp-optimization-blueprint-micro03|drive-research-llamacpp-optimization-blueprint-micro03]] (2026-06-09)
- Higher batch sizes increase the time-to-first-token.
- Speculative decoding can result in speedups without loss in output quality.
