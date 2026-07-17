---
name: tdd
description: Strict red-green-refactor loop — refuse done without failing then passing test evidence from tools.
triggers:
  - tdd
  - test-driven
requires_evidence: true
---

# TDD

Build or fix one vertical slice at a time with a red → green → refactor loop.

## Loop

1. **Red** — Write or identify a failing test first. Run it via `shell_exec` (`cargo test …`). Cite the failure output.
2. **Green** — Make the smallest change that passes. Re-run the same test. Cite the pass output.
3. **Refactor** — Clean up only with tests still green. Re-run if you touched logic.

## Rules

- Never claim "done" without tool-cited red-then-green evidence in this session.
- One slice per cycle. Prefer focused `cargo test <filter>` over the full suite until the slice is green.
- If you cannot reproduce a failing test, stop and diagnose — do not "fix forward" with untested code.
- Use `file_read` / `file_write` / `file_search` as needed; keep diffs surgical.
