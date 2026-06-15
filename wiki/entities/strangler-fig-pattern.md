---
type: entity
title: Strangler Fig pattern
created: 2026-06-08
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# Strangler Fig pattern

Type: CONCEPT

## From [[architectural-framework-for-scalable-codebase-rest|architectural-framework-for-scalable-codebase-rest]] (2026-06-08)
- It is a pattern that can be used to gradually redirect traffic when transitioning from a modular monolith to microservices.
- It is akin to the Branch by Abstraction technique.
- It allows for incremental migration with the ability to swiftly toggle back to the old implementation.

## From [[google-antigravity-the-architects-configuration-micro06|google-antigravity-the-architects-configuration-micro06]] (2026-06-09)
- Angewendet bei kritischen Modulen in Schritt 2 der Migration.
- Kombiniert mit Canary-Releases.
