---
type: entity
title: Discretionary Access Control (DAC) locking
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Discretionary Access Control (DAC) locking

Type: CONCEPT

## From [[architectural-analysis-of-the-openclaw-ai-plugin-s|architectural-analysis-of-the-openclaw-ai-plugin-s]] (2026-06-08)
- A crucial security parameter applied in highly hardened enterprise environments.
- Involves executing a chown command on the .openclaw directory tree.
- Applies a sticky bit protocol (chmod 1755) to the .openclaw directory.
