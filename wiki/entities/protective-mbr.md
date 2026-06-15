---
type: entity
title: Protective MBR
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Protective MBR

Type: CONCEPT

## From [[architecting-the-minimalist-linux-desktop-a-compa-part2|architecting-the-minimalist-linux-desktop-a-compa-part2]] (2026-06-08)
- Should contain a single 0xEE partition according to the GPT specification.
- Its only purpose is protective.
- Buggy motherboards refuse to pass execution control to a disk unless they detect an MBR partition marked with the traditional 0x80 active/boot flag.
