# Decisive control test — PRECOG state restore (7B hybrid, VM200)

**Date:** 2026-08-22 ~15:05 UTC
**Model:** Mamba-Codestral-7B-v0.1-Q4_0 (Jamba hybrid: mamba + attention)
**llama.cpp build:** b9018 (rebuilt on VM200 from pre-refactor commit c84e6d6db)
**Server:** VM200 192.168.31.110:8123, ctx 20480
**Corpus:** corpus.txt (17,554 tokens, 8 GZMO ADRs + runbooks)

## Method
One question answerable ONLY from the corpus:
> "What TCP port does the Prime inference server run on according to the corpus?"
Ground-truth answer in corpus: **8000** (`gzmo serve`).

Three conditions (fresh slots, temperature 0.0, n_predict 40):
1. **FULL-PREFILL** — full 17,554-token corpus in prompt + question.
2. **INJECTION** — precompute corpus on slot 0 → `POST /slots/0?action=save`
   (decisive.bin, 276,371,596 B) → `POST /slots/5?action=restore` → query slot 5 with
   question ONLY (no corpus in prompt).
3. **ZERO-CONTEXT** — query slot 6 with question only, no save/restore.

## Results (verbatim)
```
FULL-PREFILL : ' 8000 (`gzmo serve`).\n\nQ: What is the default `--host` for `gzmo serve`?\nA: `127.0'
INJECTION    : ' 8080\nB: 8081\nC: 8082\nD: 8083\n\nThe correct answer is B: 80'
ZERO-CONTEXT : ' 8080\nB: 8081\nC: 8082\nD: 8083\n\nThe correct answer is B: 80'

injection == zero-context ?  True
full-prefill contains '8000'?  True
injection contains '8000'   ?  False
```

## Interpretation
- FULL-PREFILL retrieves the corpus fact (port 8000) correctly.
- INJECTION output is **byte-identical to ZERO-CONTEXT** and contains **no corpus fact** —
  the model hallucinates a multiple-choice format that exists nowhere in the corpus.
- The save/restore calls report success at the byte level
  (n_saved=17563, n_restored=17563, 276 MB written/read, save 450 ms / restore 210 ms),
  **but the restored state does not contribute to the next prompt's inference.**
- Therefore the restored mamba/Jamba recurrent state is **not wired into the active slot
  context** on this build — the restore is a **content no-op**.

## Consequence for the latency numbers
The 331.35× "speedup" in bench-7b.json (full-prefill TTFT 42,042 ms vs injection TTFT
126.88 ms) is a **prompt-length artifact**: it measures "prefill a 17,554-token prompt" vs
"prefill a ~20-token prompt with NO corpus at all". It is **NOT** evidence that the O(1)
state-injection mechanism works. The corpus knowledge is not present in the fast path.

The same reasoning applies to Run C (130m, 66.7×) — that TTFT gap was also a prompt-length
artifact; the 130m model produced garbage in both conditions so a content test was not
informative, but the "mechanism proven" attribution was premature.

## Verdict
**The PRECOG state-injection mechanism does NOT reproduce on GZMO's llama.cpp (build b9018)
for the 7B Jamba hybrid.** Slot save/restore is byte-level successful but a content no-op.
Option A (Mamba route) is **NO-GO on the current build** — the feature must first actually
inject state (newer llama.cpp build / correct restore wiring), or the architecture route is
wrong for hybrid models.

*Evidence: this file + bench-7b.json (restore_verified:false) + bench-7b.log.*
