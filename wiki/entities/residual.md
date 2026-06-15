---
type: entity
title: ResiDual
created: 2026-06-09
updated: 2026-06-09
sources: 3
tags: []
status: draft
gzmo_synthetic: true
---




# ResiDual

Type: CONCEPT

## From [[ai-research-part6-micro02|ai-research-part6-micro02]] (2026-06-09)
- A Post-Norm variant with an additional shortcut from each block to the network output.
- Exhibits better stability than vanilla Post-Norm but suffers from frequent loss spikes.
- Suffers from frequent loss spikes.

## From [[ai-research-part6-micro03|ai-research-part6-micro03]] (2026-06-09)
- Mitigates Post-Norm instability via per-block shortcuts to the output.
- A topology-modifying work.

## From [[ai-research-part6-micro04|ai-research-part6-micro04]] (2026-06-09)
- Work most structurally similar to SiameseNorm
- Pre-Norm stream is not connected to the input of the residual block
- Published in 2023
