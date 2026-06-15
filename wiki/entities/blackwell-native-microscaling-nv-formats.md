---
type: entity
title: Blackwell Native Microscaling (NV Formats)
created: 2026-06-08
updated: 2026-06-08
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# Blackwell Native Microscaling (NV Formats)

Type: CONCEPT

## From [[drive-research-blackwell-sm120-gemm-optimization-guide|drive-research-blackwell-sm120-gemm-optimization-guide]] (2026-06-08)
- Designed to maximize computational throughput.
- nv_float4_t format implements float_ue4m3_t scale factor type.
- Scaling granularity is halved to SV of 16 elements.

## From [[drive-research-what-else-can-directly-be-aligned-with-our-common|drive-research-what-else-can-directly-be-aligned-with-our-common]] (2026-06-08)
- Allows leveraging the more precise float_ue4m3_t scale factor format.
- Effectively halves scaling granularity to an SV=16 basic-block configuration.
- Dramatically enhancing representational accuracy for highly skewed weight distributions without adding a single byte to physical VRAM footprint.
