---
type: entity
title: KI-Pipeline
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# KI-Pipeline

Type: CONCEPT

## From [[aether-grid-micro02|aether-grid-micro02]] (2026-06-09)
- Input: DeepStream (Person <1 m) + Riva ASR.
- Spark: TensorRT-LLM Instinct-Threshold -> Level 0/1 local.
- Core-Sync: Anonymized Intent Vector -> Qdrant Namespace -> Triton-Response.
- Action: Home Assistant -> SNMP/IPP/KNX + DALI-Light.
- Output: Riva TTS + visual feedback (Light-Pulse).
