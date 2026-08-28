#!/usr/bin/env python3
"""bench-7b.py — Pure Python PRECOG 7B bench for Mamba-Codestral-7B on VM200.
Full-prefill baseline vs state-injection, 5 questions × 5 runs each.
Server: VM200 http://192.168.31.110:8123 (already running).
"""
import json
import os
import statistics
import urllib.request
import subprocess

HOST = "192.168.31.110"
PORT = 8123
URL = f"http://{HOST}:{PORT}"
SSH = ["ssh", "-i", os.path.expanduser("~/.ssh/id_sidecar_proxmox"), "maximilian@192.168.31.110"]

SPIKE_DIR = os.path.dirname(os.path.abspath(__file__))
CORPUS_FILE = os.path.join(SPIKE_DIR, "corpus.txt")
STATE_DIR = "/opt/models/pre-cog"
STATE_FILE = "bench-7b-state.bin"

CORPUS_FILES = "docs/adr/ADR-0003-one-instance-metabolism.md docs/adr/ADR-0004-airgap-living-usp.md docs/adr/ADR-0005-flywheel-over-frozen-topology.md docs/adr/ADR-0006-owner-control-plane.md docs/adr/ADR-0007-one-product-living.md docs/adr/ADR-0008-edge-ssm-memory.md docs/GZMO_NEXT_RUNBOOK.md docs/ops/PI_UPGRADE_RUNBOOK.md"

QUESTIONS = [
    "What does ADR-0004 describe as the key USP of the airgapped living system?",
    "What is the flywheel approach in ADR-0005 replacing?",
    "What process lock mechanism does ADR-0006 use for the living writer?",
    "What does ADR-0007 say about the lite SKU?",
    "What does ADR-0008 describe regarding edge SSM memory?",
]


def api_post(path, data, timeout=180):
    payload = json.dumps(data).encode("utf-8")
    req = urllib.request.Request(
        f"{URL}{path}", data=payload, headers={"Content-Type": "application/json"}
    )
    with urllib.request.urlopen(req, timeout=timeout) as resp:
        return json.loads(resp.read())


def api_get(path, timeout=30):
    req = urllib.request.Request(f"{URL}{path}")
    with urllib.request.urlopen(req, timeout=timeout) as resp:
        return json.loads(resp.read())


def main():
    # Verify server
    health = api_get("/health")
    print(f"Server health: {health}")

    # Tokenize corpus
    with open(CORPUS_FILE) as f:
        corpus = f.read()
    tok_resp = api_post("/tokenize", {"content": corpus})
    n_tokens = len(tok_resp["tokens"])
    print(f"Corpus tokens: {n_tokens}")

    # === (a) FULL-PREFILL: each question with full corpus in prompt ===
    print("\n=== (a) FULL-PREFILL (5 questions × 5 runs) ===")
    full_ttfts = []
    full_tps = []
    full_answers = []

    for qi, q in enumerate(QUESTIONS):
        for run in range(5):
            resp = api_post("/completion", {
                "prompt": f"Context:\n{corpus}\n\nQ: {q}\nA:",
                "n_predict": 128,
                "stream": False,
                "cache_prompt": False,
                "temperature": 0.0,
            }, timeout=300)
            t = resp["timings"]
            ttft = round(t.get("prompt_ms", 0) + 0.001, 3)
            tps = round(t.get("predicted_per_second", 0), 2)
            ans = resp.get("content", "")[:300]
            full_ttfts.append(ttft)
            full_tps.append(tps)
            full_answers.append(ans)
            print(f"  Q{qi+1} run {run+1}: TTFT={ttft}ms tok/s={tps}")

    full_median = statistics.median(full_ttfts)
    full_tps_median = statistics.median(full_tps)
    print(f"FULL-PREFILL median TTFT: {full_median}ms, median tok/s: {full_tps_median}")

    # === (b) PRECOMPUTE: feed corpus into slot 0, save state ===
    print("\n=== (b) PRECOMPUTE corpus into slot 0 ===")
    api_post("/completion", {
        "prompt": f"Context:\n{corpus}\n\nReply: OK",
        "n_predict": 1,
        "cache_prompt": True,
        "stream": False,
        "temperature": 0.0,
        "id_slot": 0,
    }, timeout=600)
    print("precompute done")

    # Save state from slot 0
    save_resp = api_post("/slots/0?action=save", {"filename": STATE_FILE}, timeout=60)
    print(f"save: {save_resp}")
    n_saved = save_resp.get("n_saved", 0)

    # Get state file size
    result = subprocess.run(SSH + [f"stat -c%s {STATE_DIR}/{STATE_FILE} 2>/dev/null || echo 0"],
                           capture_output=True, text=True)
    state_size = int(result.stdout.strip())

    # === Restore verification ===
    print("\n=== Restore verification ===")
    restore_resp = api_post("/slots/0?action=restore", {"filename": STATE_FILE}, timeout=60)
    print(f"restore: {restore_resp}")
    n_restored = restore_resp.get("n_restored", 0)

    # Injection test
    inj_test = api_post("/completion", {
        "prompt": "Q: What does ADR-0004 describe as the key USP of the airgapped living system?\nA:",
        "n_predict": 64,
        "stream": False,
        "cache_prompt": True,
        "temperature": 0.0,
        "id_slot": 0,
    }, timeout=60)
    inj_test_ans = inj_test.get("content", "")[:200]
    print(f"injection test answer: {inj_test_ans}")

    # Zero-context control
    zero_test = api_post("/completion", {
        "prompt": "Q: What does ADR-0004 describe as the key USP of the airgapped living system?\nA:",
        "n_predict": 64,
        "stream": False,
        "cache_prompt": False,
        "temperature": 0.0,
        "id_slot": 1,
    }, timeout=60)
    zero_test_ans = zero_test.get("content", "")[:200]
    print(f"zero-context test answer: {zero_test_ans}")

    # Also check: does the full-prefill answer differ from zero-context?
    # If all three produce the same answer, the model may be ignoring context entirely
    full_test = full_answers[0][:200]
    print(f"full-prefill test answer: {full_test}")

    if inj_test_ans != zero_test_ans:
        restore_verified = True
        print("RESTORE VERIFIED: injection answer differs from zero-context")
    else:
        restore_verified = False
        print("RESTORE NOT VERIFIED: injection answer same as zero-context (restore may be no-op or model ignores injected state)")

    # === (c) INJECTION: restore state before each run, query without corpus ===
    print("\n=== (c) INJECTION (5 questions × 5 runs) ===")
    inj_ttfts = []
    inj_answers = []

    for qi, q in enumerate(QUESTIONS):
        for run in range(5):
            # Restore state before each query
            api_post("/slots/0?action=restore", {"filename": STATE_FILE}, timeout=60)

            resp = api_post("/completion", {
                "prompt": f"Q: {q}\nA:",
                "n_predict": 128,
                "stream": False,
                "cache_prompt": True,
                "temperature": 0.0,
                "id_slot": 0,
            }, timeout=120)
            t = resp["timings"]
            ttft = round(t.get("prompt_ms", 0) + 0.001, 3)
            ans = resp.get("content", "")[:300]
            inj_ttfts.append(ttft)
            inj_answers.append(ans)
            print(f"  Q{qi+1} run {run+1}: TTFT={ttft}ms")

    inj_median = statistics.median(inj_ttfts)
    print(f"INJECTION median TTFT: {inj_median}ms")

    speedup = round(full_median / max(inj_median, 0.001), 2)
    print(f"SPEEDUP RATIO: {speedup}x")

    # Per-question medians
    full_per_q = [statistics.median(full_ttfts[i*5:(i+1)*5]) for i in range(5)]
    inj_per_q = [statistics.median(inj_ttfts[i*5:(i+1)*5]) for i in range(5)]

    # Representative answers (run 1 of each question)
    full_ans_q = [full_answers[i*5] for i in range(5)]
    inj_ans_q = [inj_answers[i*5] for i in range(5)]

    bench = {
        "model": "Mamba-Codestral-7B-v0.1-Q4_0",
        "model_path": "/opt/models/pre-cog/Mamba-Codestral-7B-v0.1-Q4_0.gguf",
        "llama_cpp_build": "b9018 (c84e6d6db) — rebuilt from pre-refactor commit on VM200; build 9378 has mamba2 tensor shape regression",
        "corpus_tokens": n_tokens,
        "corpus_files": CORPUS_FILES,
        "state_size_bytes": state_size,
        "state_n_saved": n_saved,
        "state_n_restored": n_restored,
        "restore_verified": restore_verified,
        "full_prefill": {
            "ttft_ms_median": full_median,
            "tok_per_s_median": full_tps_median,
            "ttft_ms_runs": full_ttfts,
            "ttft_per_question_ms": full_per_q,
            "answers": full_ans_q,
        },
        "injection": {
            "ttft_ms_median": inj_median,
            "ttft_ms_runs": inj_ttfts,
            "ttft_per_question_ms": inj_per_q,
            "answers": inj_ans_q,
        },
        "speedup_ratio": speedup,
        "runs_per_question": 5,
        "total_runs": 25,
        "questions": QUESTIONS,
        "caveats": [
            "Pascal GTX 1070 (8 GB, ~5.7 GB free for model), not RTX 5070 Ti",
            "llama.cpp build b9018 rebuilt on VM200 from pre-refactor commit c84e6d6db (build 9378 has mamba2 tensor shape regression: blk.0.ssm_in.weight wrong shape)",
            "full-prefill TTFT = prompt eval time (time to first token after processing the full corpus)",
            "injection TTFT = prompt eval time after restoring saved mamba state (corpus not in prompt)",
            "n_predict=128 for answer generation; TTFT measures prefill only",
            "temperature=0.0 for determinism",
            "5 questions × 5 runs = 25 measurements per condition; median across all 25 reported",
            "state saved via POST /slots/0?action=save after precomputing corpus with cache_prompt=true on slot 0",
            "state restored via POST /slots/0?action=restore before each injection query",
        ],
    }

    out_path = os.path.join(SPIKE_DIR, "bench-7b.json")
    with open(out_path, "w") as f:
        json.dump(bench, f, indent=2)
    print(f"\n=== bench-7b.json written to {out_path} ===")
    print(f"Full-prefill median: {full_median}ms")
    print(f"Injection median: {inj_median}ms")
    print(f"Speedup: {speedup}x")
    print(f"Restore verified: {restore_verified}")


if __name__ == "__main__":
    main()
