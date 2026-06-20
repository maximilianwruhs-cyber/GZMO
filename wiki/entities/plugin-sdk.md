---
type: entity
title: Plugin SDK
created: 2026-06-08
updated: 2026-06-10
sources: 3
tags: []
status: draft
gzmo_synthetic: true
---



# Plugin SDK

Type: TOOL

## From [architectural-analysis-of-the-openclaw-ai-plugin-s](/entities/architectural-analysis-of-the-openclaw-ai-plugin-s.md) (2026-06-08)
- Exposes the createPluginRuntimeStore primitive.
- Enables cross-system interactions through the active runtime API context.
- Allows plugins to manipulate the generic outboundAdapter.

## From [openclaw-deep-research-part10-micro06](/entities/openclaw-deep-research-part10-micro06.md) (2026-06-09)
- The public plugin contract that extensions are allowed to import.
- Runtime resolves openclaw/plugin-sdk via jiti alias.

## From [openclaw-deep-research-part10-micro07](/entities/openclaw-deep-research-part10-micro07.md) (2026-06-10)
- The only public cross-package contract for extension-facing SDK code
