---
type: entity
title: unist-util-visit-parents
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# unist-util-visit-parents

Type: TOOL

## From [high-fidelity-markdown-engineering-and-ast-process](/entities/high-fidelity-markdown-engineering-and-ast-process.md) (2026-06-08)
- Traversal utility.
- Executes depth-first tree traversal.
- Accepts tree, test condition, visitor callback, and reverse boolean.
- Visitor callback receives node, index, and parent.
- Can return CONTINUE, SKIP, or EXIT constants.
- Used to locate specific directive nodes.
- Allows transformation of directive nodes into complex HTML representations.
- Can map directive nodes to specialized React components.
- Functions identically to visit but alters visitor callback signature.
- Callback receives node and ancestors array (complete stack trace).
- Advanced traversal function.
- Used for surgically locating specific checkboxes and section headings.
- Performs precise array-splicing injections.
