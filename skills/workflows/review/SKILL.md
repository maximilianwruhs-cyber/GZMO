---
name: review
description: Dual-axis code review — Standards (repo style + smells) and Spec (does the change match the asked intent).
triggers:
  - review
  - code-review
requires_evidence: false
---

# Review

Review a diff or scoped change on two axes. Run them independently; do not let one pollute the other.

## Axis A — Standards

- Matches repo conventions (Rust style in `gzmo-core`, focused diffs, no secrets).
- Flag Fowler-style smells: shotguns, feature envy, deep nesting, dead code, god modules.
- Note missing tests only as a Standards issue when the area normally has them.

## Axis B — Spec

- Restate the intended change in one sentence.
- Check each requirement / operator ask against the actual diff.
- Flag missing pieces, overbuild, and behavior that contradicts the ask.

## Output format

1. **Summary** (2–4 lines)
2. **Standards findings** (severity: blocker / should-fix / nit)
3. **Spec findings** (same severities)
4. **Verdict**: approve / approve-with-nits / request-changes

Use `file_read`, `file_search`, and `shell_exec` (`git diff`, `cargo test`) for evidence. Prefer citing paths and line-level issues.
