---
type: entity
title: Quantized Johnson-Lindenstrauss (QJL)
created: 2026-06-08
updated: 2026-06-09
sources: 4
tags: []
status: draft
gzmo_synthetic: true
---




# Quantized Johnson-Lindenstrauss (QJL)

Type: TOOL

## From [[drive-research-speicherbandbreiten-engpass-memory-wall|drive-research-speicherbandbreiten-engpass-memory-wall]] (2026-06-08)
- The second stage of the TurboQuant algorithm.
- Acts as a mathematical error-checker and zero-bias estimator.
- Routes the quantization residual through QJL, storing only the sign bit of each resulting value as error correction data.
- Introduces a zero-bias estimator to the TurboQuant algorithm.
- Processes the quantization residual.
- Stores sign bits for error correction.

## From [[drive-research-llamacpp-optimization-blueprint-micro03|drive-research-llamacpp-optimization-blueprint-micro03]] (2026-06-09)
- An alternative to MSE-only quantization.
- Eliminates quantization bias at the cost of massive variance explosions.
- Degrades performance severely.

## From [[optimizing-nvidia-blackwell-sm120-part1-micro02|optimizing-nvidia-blackwell-sm120-part1-micro02]] (2026-06-09)
- Implementations face a choice between minimizing Mean Squared Error (MSE-only via Lloyd-Max scalar quantization) or utilizing Quantized Johnson-Lindenstrauss (QJL) transforms to eliminate quantization bias.
- QJL actually degrades performance severely.
- QJL implementation eliminates bias at the direct cost of massive variance explosions.
- This variance perturbs the individual attention scores, entirely rearranging the Top-K token ranking and destroying top-1 token consistency.

## From [[the-architecture-of-speculative-decoding-and-infer-part1-micro05|the-architecture-of-speculative-decoding-and-infer-part1-micro05]] (2026-06-09)
- The second stage of the TurboQuant pipeline.
- Acts as a mathematical error-checker for inner product estimations.
- Serves as a zero-bias estimator.
