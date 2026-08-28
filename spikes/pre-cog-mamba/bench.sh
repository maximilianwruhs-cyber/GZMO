#!/usr/bin/env bash
# PRECOG bench — full-prefill baseline vs state-injection on mamba.
# Only runs if capability gates pass (capability.json says state_save AND state_load supported).
# Self-starts llama-server on 127.0.0.1:8123 — NEVER touches :8000.
set -euo pipefail
cd "$(dirname "$0")"

LLAMA_CPP=/home/gzmo/llama.cpp
BUILD="$LLAMA_CPP/build/bin"
PORT=8123
HOST=127.0.0.1
STATE_DIR="$(pwd)"

# Pick biggest model that fits free GPU mem with >=1 GB headroom
FREE_GPU_MB=$(nvidia-smi --query-gpu=memory.free --format=csv,noheader,nounits | head -1)
echo "free GPU mem (first GPU): ${FREE_GPU_MB} MiB"

if [ "$FREE_GPU_MB" -gt 3000 ]; then
  MODEL=/home/gzmo/models/pre-cog-mamba/mamba-790m/mamba-790m-q4_0.gguf
  MODEL_NAME="mamba-790m-q4_0"
else
  MODEL=/home/gzmo/models/pre-cog-mamba/mamba-130m/mamba-130m-q8_0.gguf
  MODEL_NAME="mamba-130m-q8_0"
fi
echo "model: $MODEL_NAME ($MODEL)"

# Check capability gate
INFER=$(python3 -c "import json; print(json.load(open('capability.json'))['inference'])")
SAVE=$(python3 -c "import json; print(json.load(open('capability.json'))['state_save'])")
LOAD=$(python3 -c "import json; print(json.load(open('capability.json'))['state_load'])")

if [ "$SAVE" != "supported" ] || [ "$LOAD" != "supported" ]; then
  GATE="state_save=$SAVE state_load=$LOAD"
  echo "GATE FAILED: $GATE — skipping bench"
  cat > bench.json <<EOF
{"skipped": true, "reason": "capability gate failed: $GATE"}
EOF
  exit 0
fi

# Corpus
CORPUS_FILE=corpus.txt
CORPUS=$(cat "$CORPUS_FILE")
CORPUS_FILES="docs/adr/ADR-0003-one-instance-metabolism.md docs/adr/ADR-0004-airgap-living-usp.md docs/adr/ADR-0005-flywheel-over-frozen-topology.md docs/adr/ADR-0006-owner-control-plane.md docs/adr/ADR-0007-one-product-living.md docs/adr/ADR-0008-edge-ssm-memory.md docs/GZMO_NEXT_RUNBOOK.md docs/ops/PI_UPGRADE_RUNBOOK.md"

# Tokenize corpus (after server is ready — moved below)

# 5 questions about the corpus
QUESTIONS=(
  "What does ADR-0004 describe as the key USP of the airgapped living system?"
  "What is the flywheel approach in ADR-0005 replacing?"
  "What process lock mechanism does ADR-0006 use for the living writer?"
  "What does ADR-0007 say about the lite SKU?"
  "What does ADR-0008 describe regarding edge SSM memory?"
)

# Start our own server
pkill -f "llama-server.*--port $PORT" 2>/dev/null || true
sleep 1
"$BUILD/llama-server" --host "$HOST" --port "$PORT" -m "$MODEL" -ngl 99 \
  --slot-save-path "$STATE_DIR" > "$STATE_DIR/bench-server.log" 2>&1 &
SRV_PID=$!
trap "kill $SRV_PID 2>/dev/null || true" EXIT

for i in $(seq 1 15); do
  if curl -s -m 2 "http://$HOST:$PORT/health" | grep -q '"status":"ok"'; then break; fi
  sleep 1
done
echo "server ready on $HOST:$PORT"

# Tokenize corpus
TOKENS=$(curl -s -X POST "http://$HOST:$PORT/tokenize" -H "Content-Type: application/json" \
  -d "$(python3 -c "import json; print(json.dumps({'content':open('corpus.txt').read()}))")" \
  | python3 -c "import sys,json; print(len(json.load(sys.stdin)['tokens']))")
echo "corpus tokens: $TOKENS"

# === (a) FULL-PREFILL baseline: each question with full corpus in prompt ===
echo "=== (a) FULL-PREFILL baseline (5 runs) ==="
FULL_TTFTS=()
FULL_TPS=()
FULL_ANSWERS=()
for i in "${!QUESTIONS[@]}"; do
  Q="${QUESTIONS[$i]}"
  RESP=$(curl -s -X POST "http://$HOST:$PORT/completion" \
    -H "Content-Type: application/json" \
    -d "$(python3 -c "
import json
corpus=open('corpus.txt').read()
q='''$Q'''
print(json.dumps({'prompt':'Context:\n'+corpus+'\n\nQ: '+q+'\nA:','n_predict':64,'stream':False,'cache_prompt':False,'temperature':0.0}))
")")
  TTFT=$(echo "$RESP" | python3 -c "import sys,json; d=json.load(sys.stdin); t=d['timings']; print(round(t.get('prompt_ms',0)+0.001,3))")
  TPS=$(echo "$RESP" | python3 -c "import sys,json; d=json.load(sys.stdin); t=d['timings']; print(round(t.get('predicted_per_second',0),2))")
  ANS=$(echo "$RESP" | python3 -c "import sys,json; print(json.load(sys.stdin).get('content','')[:200])")
  FULL_TTFTS+=("$TTFT")
  FULL_TPS+=("$TPS")
  FULL_ANSWERS+=("$ANS")
  echo "  run $((i+1)): TTFT=${TTFT}ms tok/s=${TPS}"
done

FULL_MEDIAN=$(printf '%s\n' "${FULL_TTFTS[@]}" | sort -n | awk 'NR==3')
FULL_TPS_MEDIAN=$(printf '%s\n' "${FULL_TPS[@]}" | sort -n | awk 'NR==3')
echo "FULL-PREFILL median TTFT: ${FULL_MEDIAN}ms, median tok/s: ${FULL_TPS_MEDIAN}"

# === (b) PRECOMPUTE: feed corpus once, save state ===
echo "=== (b) PRECOMPUTE once ==="
RESP=$(curl -s -X POST "http://$HOST:$PORT/completion" \
  -H "Content-Type: application/json" \
  -d "$(python3 -c "import json; corpus=open('corpus.txt').read(); print(json.dumps({'prompt':'Context:\n'+corpus+'\n\nReply: OK','n_predict':1,'cache_prompt':True,'stream':False,'temperature':0.0}))")")

# Find the slot that has the corpus
SLOT=$(curl -s "http://$HOST:$PORT/slots" | python3 -c "
import sys,json
for s in json.load(sys.stdin):
    if s.get('n_prompt_tokens',0)>0:
        print(s['id']); break
")
echo "corpus loaded in slot $SLOT"

# Save state
SAVE=$(curl -s -X POST "http://$HOST:$PORT/slots/$SLOT?action=save" \
  -H "Content-Type: application/json" \
  -d '{"filename":"bench-state.bin"}')
N_SAVED=$(echo "$SAVE" | python3 -c "import sys,json; print(json.load(sys.stdin).get('n_saved',0))")
STATE_SIZE=$(stat -c%s bench-state.bin 2>/dev/null || echo 0)
echo "state saved: n_saved=$N_SAVED size=${STATE_SIZE} bytes"

# === (c) INJECTION: load saved state, run SAME 5 questions without corpus ===
echo "=== (c) INJECTION (5 runs) ==="
INJ_TTFTS=()
INJ_ANSWERS=()
for i in "${!QUESTIONS[@]}"; do
  Q="${QUESTIONS[$i]}"
  # Restore state into slot 0 before each query (fresh slot)
  curl -s -X POST "http://$HOST:$PORT/slots/0?action=restore" \
    -H "Content-Type: application/json" \
    -d '{"filename":"bench-state.bin"}' > /dev/null

  RESP=$(curl -s -X POST "http://$HOST:$PORT/completion" \
    -H "Content-Type: application/json" \
    -d "$(python3 -c "
import json
q='''$Q'''
print(json.dumps({'prompt':'Q: '+q+'\nA:','n_predict':64,'stream':False,'cache_prompt':True,'temperature':0.0}))
")" 2>/dev/null)
  TTFT=$(echo "$RESP" | python3 -c "import sys,json; d=json.load(sys.stdin); t=d['timings']; print(round(t.get('prompt_ms',0)+0.001,3))")
  ANS=$(echo "$RESP" | python3 -c "import sys,json; print(json.load(sys.stdin).get('content','')[:200])")
  INJ_TTFTS+=("$TTFT")
  INJ_ANSWERS+=("$ANS")
  echo "  run $((i+1)): TTFT=${TTFT}ms"
done

INJ_MEDIAN=$(printf '%s\n' "${INJ_TTFTS[@]}" | sort -n | awk 'NR==3')
echo "INJECTION median TTFT: ${INJ_MEDIAN}ms"

# Speedup ratio
SPEEDUP=$(python3 -c "print(round($FULL_MEDIAN / max($INJ_MEDIAN,0.001), 2))")
echo "SPEEDUP RATIO: ${SPEEDUP}x"

# Cleanup state file
rm -f bench-state.bin

# === Write bench.json ===
# Build answers arrays: write each answer to individual files, then JSON-encode
TMPDIR_ANS=$(mktemp -d)
for i in 0 1 2 3 4; do printf '%s' "${FULL_ANSWERS[$i]}" > "$TMPDIR_ANS/full_$i.txt"; done
for i in 0 1 2 3 4; do printf '%s' "${INJ_ANSWERS[$i]}"  > "$TMPDIR_ANS/inj_$i.txt"; done
FULL_ANS_JSON=$(python3 -c "
import json, os
d='$TMPDIR_ANS'
print(json.dumps([open(os.path.join(d,f'full_{i}.txt')).read() for i in range(5)]))
")
INJ_ANS_JSON=$(python3 -c "
import json, os
d='$TMPDIR_ANS'
print(json.dumps([open(os.path.join(d,f'inj_{i}.txt')).read() for i in range(5)]))
")
rm -rf "$TMPDIR_ANS"

cat > bench.json <<EOF
{
  "model": "$MODEL_NAME",
  "model_path": "$MODEL",
  "corpus_tokens": $TOKENS,
  "corpus_files": "$CORPUS_FILES",
  "state_size_bytes": $STATE_SIZE,
  "full_prefill": {
    "ttft_ms_median": $FULL_MEDIAN,
    "tok_per_s": $FULL_TPS_MEDIAN,
    "ttft_ms_runs": [$(IFS=,; echo "${FULL_TTFTS[*]}")],
    "answers": $FULL_ANS_JSON
  },
  "injection": {
    "ttft_ms_median": $INJ_MEDIAN,
    "ttft_ms_runs": [$(IFS=,; echo "${INJ_TTFTS[*]}")],
    "answers": $INJ_ANS_JSON
  },
  "speedup_ratio": $SPEEDUP,
  "runs": 5,
  "caveats": [
    "$MODEL_NAME is a small general model — quality is INDICATIVE only; the claim under test is the LATENCY mechanism, quality at 7B-class is a separate gate",
    "full-prefill TTFT = prompt eval time (time to first token after processing the full corpus)",
    "injection TTFT = prompt eval time after restoring saved mamba state (corpus not in prompt)",
    "n_predict=64 for answer generation; TTFT measures prefill only",
    "GPU: 2x RTX 5070 Ti, production :8000 server (qwen3.8-27b) running concurrently",
    "temperature=0.0 for determinism"
  ]
}
EOF

echo "=== bench.json ==="
cat bench.json

# Cleanup
kill $SRV_PID 2>/dev/null || true
rm -f bench-server.log
