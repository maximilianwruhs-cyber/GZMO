---
type: entity
title: openclaw-deep-research-part1
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# openclaw-deep-research-part1

Type: PROJECT

## From [[openclaw-deep-research-part1-micro04|openclaw-deep-research-part1-micro04]] (2026-06-09)
- The main project identifier.
- Contains micro-splits for further processing.
- Ingests micro-splits for cloud KG extraction.
- Reads env vars from the parent process.
- Can set inline env vars in config.
- Runs login shell if enabled and expected keys aren't set.
- Supports env var substitution in config values.
- Supports SecretRef objects for fields.
