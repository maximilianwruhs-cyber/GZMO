---
type: entity
title: Bun runtime
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Bun runtime

Type: TOOL

## From [[drive-research-license-and-native-binding-analysis|drive-research-license-and-native-binding-analysis]] (2026-06-08)
- Used on an Ubuntu Linux target environment for the gzmo-daemon.
- Offers advantages in runtime execution speed and installation caching.
- Utilizes Apple's JavaScriptCore (JSC) engine.
- Resulting daemon artifact can be deployed reliably on minimal Ubuntu base images.
- Bindings to libraries are avoided in favor of subprocesses.
