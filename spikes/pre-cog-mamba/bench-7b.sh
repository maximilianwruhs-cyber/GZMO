#!/usr/bin/env bash
# PRECOG 7B bench — Mamba-Codestral-7B on VM200 (GTX 1070).
# Full-prefill baseline vs state-injection, 5 questions × 5 runs each.
# Server: VM200 http://192.168.31.110:8123 (already running).
set -euo pipefail
cd "$(dirname "$0")"

HOST=192.168.31.110
PORT=8123
URL="http://$HOST:$PORT"
STATE_DIR=/opt/models/pre-cog
SSH="ssh -i ~/.ssh/id_sidecar_proxmox maximilian@192.168.31.110"

# Corpus + questions (UNCHANGED from bench.sh)
CORPUS_FILES="docs/ADR-0003-one-instance-metabolism.md docs/ADR-0004-airgap-living-usp.md docs/ADR-0005-flywheel-over-frozen-topology.md docs/ADR-0006-owner-control-plane.md docs/ADR-0007-one-product-living.md docs/ADR-0008-edge-ssm-memory.md docs/GZMO_NEXT_RUNBOOK.md docs/PI_UPGRADE_RUNBOOK.md"

QUESTIONS=(
  "What does ADR-0004 describe as the key USP of the airgapped living system?"
  "What is the flywheel approach in ADR-0005 replacing?"
  "What process lock mechanism does ADR-0006 use for the living writer?"
  "What does ADR-0007 say about the lite SKU?"
  "What does ADR-0008 describe regarding edge SSM memory?"
)

# Verify server
echo "=== Server health ==="
curl -s -m 5 "$URL/health"; echo

# Tokenize corpus
TOKENS=$(curl -s -X POST "$URL/tokenize" -H "Content-Type: application/json" \
  -d "$(python3 -c "import json; print(json.dumps({'content':open('corpus.txt').read()}))")" \
  | python3 -c "import sys,json; print(len(json.load(sys.stdin)['tokens']))")
echo "corpus tokens: $TOKENS"

# === (a) FULL-PREFILL: each question with full corpus in prompt ===
echo "=== (a) FULL-PREFILL (5 questions × 5 runs) ==="
FULL_TTFTS=()
FULL_TPS=()
FULL_ANSWERS=()

for qi in "${!QUESTIONS[@]}"; do
  Q="${QUESTIONS[$qi]}"
  for run in $(seq 1 5); do
    RESP=$(curl -s -m 180 -X POST "$URL/completion" \
      -H "Content-Type: application/json" \
      -d "$(python3 -c "
import json
corpus=open('corpus.txt').read()
q='''$Q'''
print(json.dumps({'prompt':'Context:\n'+corpus+'\n\nQ: '+q+'\nA:','n_predict':128,'stream':False,'cache_prompt':False,'temperature':0.0}))
")")
    TTFT=$(echo "$RESP" | python3 -c "import sys,json; d=json.load(sys.stdin); t=d['timings']; print(round(t.get('prompt_ms',0)+0.001,3))")
    TPS=$(echo "$RESP" | python3 -c "import sys,json; d=json.load(sys.stdin); t=d['timings']; print(round(t.get('predicted_per_second',0),2))")
    ANS=$(echo "$RESP" | python3 -c "import sys,json; print(json.load(sys.stdin).get('content','')[:300])")
    FULL_TTFTS+=("$TTFT")
    FULL_TPS+=("$TPS")
    FULL_ANSWERS+=("$ANS")
    echo "  Q$((qi+1)) run $run: TTFT=${TTFT}ms tok/s=${TPS}"
  done
done

FULL_MEDIAN=$(printf '%s\n' "${FULL_TTFTS[@]}" | sort -n | awk 'NR==13')
FULL_TPS_MEDIAN=$(printf '%s\n' "${FULL_TPS[@]}" | sort -n | awk 'NR==13')
echo "FULL-PREFILL median TTFT: ${FULL_MEDIAN}ms, median tok/s: ${FULL_TPS_MEDIAN}"

# === (b) PRECOMPUTE: feed corpus into slot 0, save state ===
echo "=== (b) PRECOMPUTE corpus into slot 0 ==="
curl -s -m 600 -X POST "$URL/completion" \
  -H "Content-Type: application/json" \
  -d "$(python3 -c "import json; corpus=open('corpus.txt').read(); print(json.dumps({'prompt':'Context:\n'+corpus+'\n\nReply: OK','n_predict':1,'cache_prompt':True,'stream':False,'temperature':0.0,'id_slot':0}))")" > /dev/null
echo "precompute done"

# Save state from slot 0
echo "=== Save state from slot 0 ==="
SAVE=$(curl -s -m 60 -X POST "$URL/slots/0?action=save" \
  -H "Content-Type: application/json" \
  -d '{"filename":"bench-7b-state.bin"}')
echo "save: $SAVE"
N_SAVED=$(echo "$SAVE" | python3 -c "import sys,json; print(json.load(sys.stdin).get('n_saved',0))")
STATE_SIZE=$($SSH "stat -c%s $STATE_DIR/bench-7b-state.bin 2>/dev/null || echo 0")
echo "state: n_saved=$N_SAVED size=${STATE_SIZE} bytes"

# Verify restore works (restore_verified check)
RESTORE=$(curl -s -m 60 -X POST "$URL/slots/0?action=restore" \
  -H "Content-Type: application/json" \
  -d '{"filename":"bench-7b-state.bin"}')
echo "restore: $RESTORE"
N_RESTORED=$(echo "$RESTORE" | python3 -c "import sys,json; print(json.load(sys.stdin).get('n_restored',0))")
echo "n_restored=$N_RESTORED"

# Compare injection answer vs full-prefill answer to verify state was actually injected
INJ_TEST=$(curl -s -m 60 -X POST "$URL/completion" \
  -H "Content-Type: application/json" \
  -d '{"prompt":"Q: What does ADR-0004 describe as the key USP of the airgapped living system?\nA:","n_predict":64,"stream":false,"cache_prompt":true,"temperature":0.0,"id_slot":0}')
INJ_TEST_ANS=$(echo "$INJ_TEST" | python3 -c "import sys,json; print(json.load(sys.stdin).get('content','')[:200])")
echo "injection test answer: $INJ_TEST_ANS"

# Zero-context control (no restore, fresh slot)
ZERO_TEST=$(curl -s -m 60 -X POST "$URL/completion" \
  -H "Content-Type: application/json" \
  -d '{"prompt":"Q: What does ADR-0004 describe as the key USP of the airgapped living system?\nA:","n_predict":64,"stream":false,"cache_prompt":false,"temperature":0.0,"id_slot":1}')
ZERO_TEST_ANS=$(echo "$ZERO_TEST" | python3 -c "import sys,json; print(json.load(sys.stdin).get('content','')[:200])")
echo "zero-context test answer: $ZERO_TEST_ANS"

if [ "$INJ_TEST_ANS" != "$ZERO_TEST_ANS" ]; then
  RESTORE_VERIFIED=true
  echo "RESTORE VERIFIED: injection answer differs from zero-context"
else
  RESTORE_VERIFIED=false
  echo "RESTORE NOT VERIFIED: injection answer same as zero-context (restore may be no-op)"
fi

# === (c) INJECTION: restore state before each run, query without corpus ===
echo "=== (c) INJECTION (5 questions × 5 runs) ==="
INJ_TTFTS=()
INJ_ANSWERS=()

for qi in "${!QUESTIONS[@]}"; do
  Q="${QUESTIONS[$qi]}"
  for run in $(seq 1 5); do
    # Restore state into slot 0 before each query
    curl -s -m 60 -X POST "$URL/slots/0?action=restore" \
      -H "Content-Type: application/json" \
      -d '{"filename":"bench-7b-state.bin"}' > /dev/null

    RESP=$(curl -s -m 120 -X POST "$URL/completion" \
      -H "Content-Type: application/json" \
      -d "$(python3 -c "
import json
q='''$Q'''
print(json.dumps({'prompt':'Q: '+q+'\nA:','n_predict':128,'stream':False,'cache_prompt':True,'temperature':0.0,'id_slot':0}))
")" 2>/dev/null)
    TTFT=$(echo "$RESP" | python3 -c "import sys,json; d=json.load(sys.stdin); t=d['timings']; print(round(t.get('prompt_ms',0)+0.001,3))")
    ANS=$(echo "$RESP" | python3 -c "import sys,json; print(json.load(sys.stdin).get('content','')[:300])")
    INJ_TTFTS+=("$TTFT")
    INJ_ANSWERS+=("$ANS")
    echo "  Q$((qi+1)) run $run: TTFT=${TTFT}ms"
  done
done

INJ_MEDIAN=$(printf '%s\n' "${INJ_TTFTS[@]}" | sort -n | awk 'NR==13')
echo "INJECTION median TTFT: ${INJ_MEDIAN}ms"

SPEEDUP=$(python3 -c "print(round($FULL_MEDIAN / max($INJ_MEDIAN,0.001), 2))")
echo "SPEEDUP RATIO: ${SPEEDUP}x"

# === Write bench-7b.json ===
TMPDIR_ANS=$(mktemp -d)
for i in $(seq 0 24); do printf '%s' "${FULL_ANSWERS[$i]}" > "$TMPDIR_ANS/full_$i.txt"; done
for i in $(seq 0 24); do printf '%s' "${INJ_ANSWERS[$i]}"  > "$TMPDIR_ANS/inj_$i.txt"; done

python3 <<PYEOF
import json, os, statistics

d = "$TMPDIR_ANS"
full_ans = [open(os.path.join(d, f"full_{i}.txt")).read() for i in range(25)]
inj_ans  = [open(os.path.join(d, f"inj_{i}.txt")).read() for i in range(25)]

full_ttfts_raw = [${FULL_TTFTS[*]}]
inj_ttfts_raw  = [${INJ_TTFTS[*]}]

# Per-question medians (5 runs each)
full_per_q = [statistics.median(full_ttfts_raw[i*5:(i+1)*5]) for i in range(5)]
inj_per_q  = [statistics.median(inj_ttfts_raw[i*5:(i+1)*5]) for i in range(5)]

questions = [
    "What does ADR-0004 describe as the key USP of the airgapped living system?",
    "What is the flywheel approach in ADR-0005 replacing?",
    "What process lock mechanism does ADR-0006 use for the living writer?",
    "What does ADR-0007 say about the lite SKU?",
    "What does ADR-0008 describe regarding edge SSM memory?",
]

# One representative answer per question (run 1)
full_ans_q = [full_ans[i*5] for i in range(5)]
inj_ans_q  = [inj_ans[i*5] for i in range(5)]

bench = {
    "model": "Mamba-Codestral-7B-v0.1-Q4_0",
    "model_path": "/opt/models/pre-cog/Mamba-Codestral-7B-v0.1-Q4_0.gguf",
    "llama_cpp_build": "b9018 (c84e6d6db) — rebuilt from pre-refactor commit on VM200; build 9378 has mamba2 tensor shape regression",
    "corpus_tokens": $TOKENS,
    "corpus_files": "$CORPUS_FILES",
    "state_size_bytes": $STATE_SIZE,
    "state_n_saved": $N_SAVED,
    "state_n_restored": $N_RESTORED,
    "restore_verified": $RESTORE_VERIFIED,
    "full_prefill": {
        "ttft_ms_median": $FULL_MEDIAN,
        "tok_per_s_median": $FULL_TPS_MEDIAN,
        "ttft_ms_runs": full_ttfts_raw,
        "ttft_per_question_ms": full_per_q,
        "answers": full_ans_q
    },
    "injection": {
        "ttft_ms_median": $INJ_MEDIAN,
        "ttft_ms_runs": inj_ttfts_raw,
        "ttft_per_question_ms": inj_per_q,
        "answers": inj_ans_q
    },
    "speedup_ratio": $SPEEDUP,
    "runs_per_question": 5,
    "total_runs": 25,
    "questions": questions,
    "caveats": [
        "Pascal GTX 1070 (8 GB, ~5.7 GB free for model), not RTX 5070 Ti",
        "llama.cpp build b9018 rebuilt on VM200 from pre-refactor commit c84e6d6db (build 9378 has mamba2 tensor shape regression: blk.0.ssm_in.weight wrong shape)",
        "full-prefill TTFT = prompt eval time (time to first token after processing the full corpus)",
        "injection TTFT = prompt eval time after restoring saved mamba state (corpus not in prompt)",
        "n_predict=128 for answer generation; TTFT measures prefill only",
        "temperature=0.0 for determinism",
        "5 questions × 5 runs = 25 measurements per condition; median across all 25 reported",
        "state saved via POST /slots/0?action=save after precomputing corpus with cache_prompt=true on slot 0",
        "state restored via POST /slots/0?action=restore before each injection query"
    ]
}

with open("bench-7b.json", "w") as f:
    json.dump(bench, f, indent=2)
print("=== bench-7b.json written ===")
PYEOF

rm -rf "$TMPDIR_ANS"
echo "=== DONE ==="
