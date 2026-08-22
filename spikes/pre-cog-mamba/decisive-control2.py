#!/usr/bin/env python3
"""decisive-control2.py — PRECOG state-injection: protocol-correct decisive matrix.

Why this exists
---------------
decisive-control.md (v1) found injection output byte-identical to zero-context.
Source audit of llama.cpp (b9018) tools/server/server-context.cpp shows why that is
EXPECTED for a no-prefix query after restore:

  - restore loads the state file into the slot's ctx AND re-fills
    slot->prompt.tokens with the SAVED (corpus) tokens.
  - a new completion whose prompt shares NO prefix with the saved tokens computes
    n_past = get_common_prefix() = 0
    -> "forcing full prompt re-processing" path (PR #13194, hybrid/recurrent)
    -> question evaluated from position 0 -> restored state unreachable.

The paper's injection protocol is PREFIX CONTINUATION: restore, then send
(saved prefix + new suffix) with cache_prompt=true. Then n_past = full prefix
length and the new tokens are evaluated at positions prefix_len+, which read the
restored SSM/KV state.

All conditions run SEQUENTIALLY on slot 0 (restore overwrites slot content each
time) so the server needs only -np 1 (fits 1070 VRAM; -np 16 OOMs at 4.2 GB).

Conditions (temperature 0.0, n_predict 40, slot 0 each time):
  C1 FULL-PREFILL    corpus+question in prompt, cache_prompt=false
  C2 INJ-PREFIX      restore -> (saved_prefix + suffix), cache_prompt=true  <-- THE mechanism test
  C3 INJ-PREFIX-NC   restore -> same prompt, cache_prompt=false (flag control)
  C4 INJ-NOPREFIX    restore -> question only, cache_prompt=true (v1's test)
  C5 ZERO-CONTEXT    fresh (erase) -> question only, cache_prompt=false

Ground truth: corpus says Prime inference server runs on TCP port 8000 (`gzmo serve`).
Only C1 and (if the mechanism works) C2 may know this.
"""
import json
import os
import urllib.request

URL = "http://192.168.31.110:8123"
SPIKE_DIR = os.path.dirname(os.path.abspath(__file__))
STATE_FILE = "bench-7b-state.bin"  # saved by bench-7b.py precompute on slot 0
QUESTION = "What TCP port does the Prime inference server run on according to the corpus?"
GT = "8000"


def post(path, data, timeout=600):
    req = urllib.request.Request(
        f"{URL}{path}", data=json.dumps(data).encode(),
        headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=timeout) as r:
        return json.loads(r.read())


def get(path, timeout=30):
    with urllib.request.urlopen(f"{URL}{path}", timeout=timeout) as r:
        return json.loads(r.read())


def main():
    print(f"health: {get('/health')}")
    corpus = open(os.path.join(SPIKE_DIR, "corpus.txt")).read()
    n_corpus = len(post("/tokenize", {"content": corpus})["tokens"])
    pre_prompt = f"Context:\n{corpus}\n\nReply: OK"
    n_pre = len(post("/tokenize", {"content": pre_prompt})["tokens"])
    print(f"corpus tokens: {n_corpus} | saved-prefix tokens: {n_pre}")

    # Sanity: restore into slot 0 and confirm token count matches precompute.
    probe = post("/slots/0?action=restore", {"filename": STATE_FILE})
    print(f"restore probe (slot 0): {probe}")
    n_saved = probe.get("n_tokens", probe.get("n_restored", 0))
    if n_saved != n_pre:
        print(f"WARNING: saved tokens {n_saved} != prefix tokens {n_pre} "
              f"(common prefix may be shorter; C2 under-tests)")

    suffix = f"\n\nQ: {QUESTION}\nA:"
    q_only = f"Q: {QUESTION}\nA:"

    # (name, prompt, cache_prompt, need_restore)
    conds = [
        ("C1_FULL_PREFILL",  f"Context:\n{corpus}\n\nQ: {QUESTION}\nA:", False, False),
        ("C2_INJ_PREFIX",    pre_prompt + suffix,                         True,  True),
        ("C3_INJ_PREFIX_NC", pre_prompt + suffix,                         False, True),
        ("C4_INJ_NOPREFIX",  q_only,                                      True,  True),
        ("C5_ZERO_CONTEXT",  q_only,                                      False, True),
    ]

    results = {}
    for name, prompt, cache_prompt, need_restore in conds:
        if need_restore:
            post("/slots/0?action=erase", {})  # clean slot before restore (C5 = fresh)
            r = post("/slots/0?action=restore", {"filename": STATE_FILE})
            print(f"\n=== {name}: restore n_tokens={r.get('n_tokens')} ===")
        else:
            post("/slots/0?action=erase", {})
            print(f"\n=== {name}: fresh (no restore) ===")
        resp = post("/completion", {
            "prompt": prompt, "n_predict": 40, "stream": False,
            "cache_prompt": cache_prompt, "temperature": 0.0, "id_slot": 0,
        })
        t = resp.get("timings", {})
        out = resp.get("content", "")
        results[name] = {
            "cache_prompt": cache_prompt,
            "n_prompt_tokens": t.get("n_prompt_tokens"),
            "n_prompt_tokens_cached": resp.get("n_prompt_tokens_cached"),
            "prompt_ms": t.get("prompt_ms"),
            "contains_gt": GT in out,
            "output": out,
        }
        print(f"{name}: prompt_tokens={t.get('n_prompt_tokens')} "
              f"cached={resp.get('n_prompt_tokens_cached')} "
              f"prompt_ms={t.get('prompt_ms')} contains_8000={GT in out}")
        print(f"  out: {out[:220]!r}")

    o = {k: results[k]["output"] for k in results}
    summary = {
        "mechanism_works": results["C2_INJ_PREFIX"]["contains_gt"],
        "c2_differs_from_zero": o["C2_INJ_PREFIX"] != o["C5_ZERO_CONTEXT"],
        "c2_matches_full_prefill_fact":
            results["C2_INJ_PREFIX"]["contains_gt"] and results["C1_FULL_PREFILL"]["contains_gt"],
        "c3_equals_c4": o["C3_INJ_PREFIX_NC"] == o["C4_INJ_NOPREFIX"],
        "c4_equals_c5": o["C4_INJ_NOPREFIX"] == o["C5_ZERO_CONTEXT"],
        "c2_prompt_ms": results["C2_INJ_PREFIX"]["prompt_ms"],
        "c1_prompt_ms": results["C1_FULL_PREFILL"]["prompt_ms"],
    }
    print("\n=== SUMMARY ===")
    for k, v in summary.items():
        print(f"  {k}: {v}")

    with open(os.path.join(SPIKE_DIR, "decisive-control2.json"), "w") as f:
        json.dump({"corpus_tokens": n_corpus, "prefix_tokens": n_pre,
                   "conditions": results, "summary": summary}, f, indent=2)
    print("\nwritten: decisive-control2.json")


if __name__ == "__main__":
    main()
