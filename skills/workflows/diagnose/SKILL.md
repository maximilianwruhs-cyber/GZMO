---
name: diagnose
description: Disciplined diagnosis for hard bugs and regressions — reproduce, minimise, hypothesise, instrument, fix, regression-test.
triggers:
  - diagnose
  - debug
  - diagnosing-bugs
requires_evidence: true
---

# Diagnose

Hard bugs and performance regressions need a disciplined loop — not guess-and-patch.

## Loop

1. **Reproduce** — Get a reliable failing case. Capture exact command/output via tools.
2. **Minimise** — Shrink input/steps until the failure is as small as possible.
3. **Hypothesise** — State 1–3 falsifiable hypotheses. Rank by likelihood.
4. **Instrument** — Add the smallest probe (log, assert, test) that distinguishes hypotheses.
5. **Fix** — Change only what the evidence supports.
6. **Regression-test** — Add or extend a test that fails without the fix and passes with it. Cite tool output.

## Rules

- Do not ship a fix without reproduce + regression evidence.
- Prefer `cargo test` / focused binaries over speculative large refactors.
- If blocked, report which step failed and what evidence is missing.
