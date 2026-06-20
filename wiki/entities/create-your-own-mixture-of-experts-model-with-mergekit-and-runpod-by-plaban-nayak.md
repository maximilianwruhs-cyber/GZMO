---
type: entity
title: Create Your Own Mixture of Experts Model with Mergekit and Runpod | by Plaban Nayak
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Create Your Own Mixture of Experts Model with Mergekit and Runpod | by Plaban Nayak

Type: TOOL

## From [architectural-blueprints-for-sovereign-frankenmoe-part1](/entities/architectural-blueprints-for-sovereign-frankenmoe-part1.md) (2026-06-08)
- The standard tooling for model merging in the open-source community.
- Allows developers to perform weight-space merges on consumer hardware without GPU training costs.
- Merges are executed using a YAML configuration file.
- Standard tooling for model merging in the open-source community.
- Used with YAML configurations for mergekit-moe.
- It addresses the challenge of initializing the gating network in zero-shot MoE fusion.
- It implements three distinct routing gate initialization modes.
- It manages vocabulary mismatches and adjusts embedding matrices during model merging.
- A specific configuration or mode within mergekit.
- Used with YAML configurations for multi-expert setups.
- Supports `gate_mode: hidden` and `shared_experts`.
- It is a command-line routine used for the compilation of a homogeneous FrankenMoE.
- It is used for weight-space stream-loading.
- It is the core script for MoE synthesis.
- It is a source.
- It is an article by Plaban Nayak.
- It discusses creating MoE models with Mergekit and Runpod.
- It is a GitHub repository.
- It contains tools for merging pretrained large language models.
- It is listed as a source.
