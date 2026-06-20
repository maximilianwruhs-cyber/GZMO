---
type: entity
title: Theiler window
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Theiler window

Type: CONCEPT

## From [drive-research-financial-time-series-lyapunov-exponents](/entities/drive-research-financial-time-series-lyapunov-exponents.md) (2026-06-08)
- A minimum time separation parameter.
- Enforces temporal isolation for nearest neighbor search.
- Ensures that if a reference vector is at index i, any candidate neighbor at index j must satisfy |i - j| > min_tsep.
