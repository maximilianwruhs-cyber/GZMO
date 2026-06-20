---
type: entity
title: TUI Component interface
created: 2026-06-08
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# TUI Component interface

Type: CONCEPT

## From [drive-research-building-pi-coding-agent-extensions](/entities/drive-research-building-pi-coding-agent-extensions.md) (2026-06-08)
- Both renderCall and renderResult functions are strictly required to return an object that implements this interface.
- Mandates implementation of specific methods like render(), handleInput?(), and invalidate?().
- Requires render() to return an array of strings representing terminal lines.
- Has a constraint that no string line may exceed the width parameter.

## From [high-performance-typescript-execution-and-architec-part1-micro06](/entities/high-performance-typescript-execution-and-architec-part1-micro06.md) (2026-06-09)
- Every UI element injected into the Pi framework must conform to this interface.
- Mandates implementation of render, handleInput, invalidate, and wantsKeyRelease methods.
- The render function must return an array of strings not exceeding the specified width.
