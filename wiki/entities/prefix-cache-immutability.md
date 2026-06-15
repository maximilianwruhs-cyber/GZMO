---
type: entity
title: prefix cache immutability
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---


# prefix cache immutability

Type: CONCEPT

## From [[drive-research-benchmarking-llamacpp-server-prefill-tokens-micro02|drive-research-benchmarking-llamacpp-server-prefill-tokens-micro02]] (2026-06-09)
- Protecting prefix cache immutability prevents expensive prompt re-evaluation passes on subsequent multi-turn requests.
- Ordering prompt payloads to protect prefix cache immutability is recommended.
- Dynamic variables, timestamps, and user turns must be appended strictly at the end of the sequence to protect it.
