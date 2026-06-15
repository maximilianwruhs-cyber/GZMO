---
type: entity
title: Pipeline Parallelism (PP)
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Pipeline Parallelism (PP)

Type: CONCEPT

## From [[drive-research-optimizing-qwen36-on-blackwell-gpus|drive-research-optimizing-qwen36-on-blackwell-gpus]] (2026-06-08)
- Partitions the model's layers sequentially.
- Communication is restricted to passing boundary activation tensors between stages.
- Running a pipeline-parallel size of 2 (PP=2) is recommended.
