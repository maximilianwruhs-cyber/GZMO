#!/usr/bin/env python3
"""final-7b.py — PRECOG 7B: airtight control + quality parity under the CORRECT protocol.

Protocol correction (see decisive-control2.json): the v1 "injection" (restore then a
no-prefix question) is n_past=0 -> SSM reset -> zero-context. That was a test error,
NOT a mechanism failure. The correct PRECOG injection is PREFIX CONTINUATION:
erase slot -> restore saved state -> send (saved_prefix + new_question) with
cache_prompt=true, so n_past = prefix_len and only the new question is computed,
reading the restored SSM/KV state.

This script closes two remaining gaps:

  (1) COLD-CONTROL (no restore): erase, then (corpus+question) with cache_prompt=true
      as the FIRST request on an empty slot. n_past=0 -> full prefill. If this is ~41s
      while C2 (restore + prefix-continue) is ~0.5s, the speedup is provably due to the
      restored file state, not a lingering in-memory KV.

  (2) QUALITY PARITY: run the same 5 ADR questions under (a) full-prefill and
      (b) correct-protocol injection; compare answers to the valid full-prefill answers
      already in bench-7b.json.
"""
import json
import os
import statistics
import urllib.request

URL = "http://192.168.31.110:8123"
SPIKE_DIR = os.path.dirname(os.path.abspath(__file__))
STATE_FILE = "bench-7b-state.bin"
CORPUS = open(os.path.join(SPIKE_DIR, "corpus.txt")).read()
PRE = f"Context:\n{CORPUS}\n\nReply: OK"          # must match saved prefix exactly
QUESTIONS = [
    "What does ADR-0004 describe as the key USP of the airgapped living system?",
    "What is the flywheel approach in ADR-0005 replacing?",
    "What process lock mechanism does ADR-0006 use for the living writer?",
    "What does ADR-0007 say about the lite SKU?",
    "What does ADR-0008 describe regarding edge SSM memory?",
]


def post(path, data, timeout=600):
    req = urllib.request.Request(f"{URL}{path}", data=json.dumps(data).encode(),
                                 headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=timeout) as r:
        return json.loads(r.read())


def get(path, timeout=30):
    with urllib.request.urlopen(f"{URL}{path}", timeout=timeout) as r:
        return json.loads(r.read())


def main():
    print(f"health: {get('/health')}")
    n_pre = len(post("/tokenize", {"content": PRE})["tokens"])
    print(f"saved-prefix tokens: {n_pre}")

    out = {}

    # ---- (1) COLD-CONTROL: no restore, cache_prompt=true, first request on empty slot
    post("/slots/0?action=erase", {})
    q0 = QUESTIONS[0]
    r = post("/completion", {
        "prompt": PRE + f"\n\nQ: {q0}\nA:", "n_predict": 40, "stream": False,
        "cache_prompt": True, "temperature": 0.0, "id_slot": 0,
    })
    cold_ms = r.get("timings", {}).get("prompt_ms")
    cold_ans = r.get("content", "")
    print(f"\nCOLD-CONTROL (no restore): prompt_ms={cold_ms} knows_8000={'8000' in cold_ans}")
    print(f"  out: {cold_ans[:160]!r}")
    out["cold_control"] = {"prompt_ms": cold_ms, "knows_8000": "8000" in cold_ans,
                           "output": cold_ans}

    # ---- (2) QUALITY PARITY, 5 questions
    full = []   # full-prefill (valid)
    inj = []    # correct-protocol injection
    for i, q in enumerate(QUESTIONS):
        # full-prefill
        post("/slots/0?action=erase", {})
        rf = post("/completion", {
            "prompt": f"Context:\n{CORPUS}\n\nQ: {q}\nA:", "n_predict": 128,
            "stream": False, "cache_prompt": False, "temperature": 0.0, "id_slot": 0,
        })
        f_ms = rf.get("timings", {}).get("prompt_ms")
        f_ans = rf.get("content", "")[:300]
        full.append({"prompt_ms": f_ms, "answer": f_ans})
        print(f"\nQ{i+1} FULL: {f_ms}ms")
        print(f"  {f_ans[:160]!r}")

        # correct-protocol injection: erase -> restore -> (prefix + question) + cache_prompt=true
        post("/slots/0?action=erase", {})
        post("/slots/0?action=restore", {"filename": STATE_FILE})
        ri = post("/completion", {
            "prompt": PRE + f"\n\nQ: {q}\nA:", "n_predict": 128, "stream": False,
            "cache_prompt": True, "temperature": 0.0, "id_slot": 0,
        })
        i_ms = ri.get("timings", {}).get("prompt_ms")
        i_ans = ri.get("content", "")[:300]
        inj.append({"prompt_ms": i_ms, "answer": i_ans})
        print(f"Q{i+1} INJ : {i_ms}ms")
        print(f"  {i_ans[:160]!r}")

    f_med = statistics.median([x["prompt_ms"] for x in full if x["prompt_ms"]])
    i_med = statistics.median([x["prompt_ms"] for x in inj if x["prompt_ms"]])
    out["quality_parity"] = {
        "full_prefill": full,
        "injection": inj,
        "full_ttft_ms_median": f_med,
        "inj_ttft_ms_median": i_med,
        "speedup": round(f_med / max(i_med, 0.001), 2),
    }
    print(f"\nFULL median TTFT: {f_med}ms | INJ median TTFT: {i_med}ms | speedup {out['quality_parity']['speedup']}x")

    with open(os.path.join(SPIKE_DIR, "final-7b.json"), "w") as f:
        json.dump(out, f, indent=2)
    print("\nwritten: final-7b.json")


if __name__ == "__main__":
    main()
