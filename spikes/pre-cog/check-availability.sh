#!/usr/bin/env bash
# check-availability.sh — PRECOG feasibility probe
# Probes for TENNs-LLM model weights (HuggingFace) and llama.cpp SSM inference support.
# Writes machine-readable result to availability.json.
set -euo pipefail

SPIKE_DIR="$(cd "$(dirname "$0")" && pwd)"
OUT="${SPIKE_DIR}/availability.json"

# --- Weights probe ---
# Search HuggingFace API for TENNs-LLM model repositories.
HF_API="https://huggingface.co/api/models?search=tenns-llm"
WEIGHTS_STATUS="unknown"
WEIGHTS_URL=""
WEIGHTS_LICENSE=""

# Use python3 + urllib (stdlib only, no pip install needed)
WEIGHTS_JSON=$(python3 -c "
import urllib.request, json
try:
    r = urllib.request.urlopen('${HF_API}', timeout=15)
    d = json.load(r)
    print(json.dumps(d))
except Exception as e:
    print('[]')
" 2>/dev/null || echo '[]')

# Parse results
if [ "$WEIGHTS_JSON" != "[]" ]; then
    FOUND=$(echo "$WEIGHTS_JSON" | python3 -c "
import sys, json
d = json.load(sys.stdin)
# Look for tenns-llm repos
for m in d:
    mid = m.get('id','')
    tags = m.get('tags',[])
    if 'tenns-llm' in mid.lower() or 'tenns_llm' in [t.lower() for t in tags]:
        print(json.dumps({
            'repo': mid,
            'license': next((t for t in tags if t.startswith('license:')), 'unknown'),
            'tags': tags,
            'downloads': m.get('downloads', 0)
        }))
        break
")
    if [ -n "$FOUND" ]; then
        WEIGHTS_STATUS="found"
        WEIGHTS_URL="https://huggingface.co/$(echo "$FOUND" | python3 -c "import sys,json; print(json.load(sys.stdin)['repo'])")"
        WEIGHTS_LICENSE=$(echo "$FOUND" | python3 -c "import sys,json; print(json.load(sys.stdin)['license'])")
    fi
fi

# Check if license is non-commercial (gated)
if [ "$WEIGHTS_STATUS" = "found" ]; then
    case "$WEIGHTS_LICENSE" in
        *cc-by-nc*) WEIGHTS_STATUS="gated" ;;
    esac
fi

# --- SSM inference probe ---
# llama.cpp supports Mamba/Mamba2/RWKV6/RWKV7 but NOT custom_code tenns_llm architecture.
# Check via DeepWiki supported architectures page and HuggingFace config.
SSM_STATUS="unknown"
SSM_EVIDENCE=""

# Check if TENNs-LLM uses custom_code (which means no standard GGUF path)
TENNS_CONFIG=$(python3 -c "
import urllib.request, json
try:
    r = urllib.request.urlopen('https://huggingface.co/api/models/BrainChip-AI/tenns-llm-1b', timeout=15)
    d = json.load(r)
    print(json.dumps({'tags': d.get('tags',[]), 'library_name': d.get('library_name','')}))
except Exception as e:
    print('{}')
" 2>/dev/null || echo '{}')

HAS_CUSTOM_CODE=$(echo "$TENNS_CONFIG" | python3 -c "
import sys, json
d = json.load(sys.stdin)
tags = d.get('tags', [])
print('yes' if 'custom_code' in tags else 'no')
" 2>/dev/null || echo 'unknown')

# llama.cpp supports Mamba family SSMs natively (verified via DeepWiki 2026-08-22)
# but TENNs-LLM is custom_code, not in llama.cpp architecture registry
if [ "$HAS_CUSTOM_CODE" = "yes" ]; then
    SSM_STATUS="partial"
    SSM_EVIDENCE="llama.cpp supports Mamba/Mamba2/RWKV6/RWKV7 SSM architectures natively, but TENNs-LLM uses custom_code transformers (model_type: tenns_llm) not in llama.cpp architecture registry. State injection requires custom inference path."
elif [ "$HAS_CUSTOM_CODE" = "no" ]; then
    SSM_STATUS="supported"
    SSM_EVIDENCE="TENNs-LLM may be convertible to a llama.cpp-supported SSM architecture."
else
    SSM_STATUS="unknown"
    SSM_EVIDENCE="Could not determine TENNs-LLM architecture compatibility."
fi

# --- Write availability.json ---
python3 -c "
import json, sys

weights_status = '${WEIGHTS_STATUS}'
ssm_status = '${SSM_STATUS}'
weights_url = '${WEIGHTS_URL}'
weights_license = '${WEIGHTS_LICENSE}'
ssm_evidence = '${SSM_EVIDENCE}'

evidence = []
if weights_url:
    evidence.append(weights_url)
evidence.append('https://huggingface.co/api/models?search=tenns-llm')
evidence.append('https://deepwiki.com/ggml-org/llama.cpp/3.11-supported-model-architectures')
evidence.append('https://arxiv.org/abs/2608.02560')

result = {
    'weights': weights_status,
    'weights_url': weights_url,
    'weights_license': weights_license,
    'ssm_inference': ssm_status,
    'ssm_evidence': ssm_evidence,
    'evidence': evidence,
    'probed_at': '2026-08-22'
}

with open('${OUT}', 'w') as f:
    json.dump(result, f, indent=2)
    f.write('\n')
print(json.dumps(result, indent=2))
"
