#!/usr/bin/env bash
# latency-bench.sh — PRECOG latency benchmark (STUB-ok)
# If availability.json says weights found AND inference supported, run real bench.
# Otherwise exit 0 with a "skipped" line and write bench.json with skipped=true.
set -euo pipefail

SPIKE_DIR="$(cd "$(dirname "$0")" && pwd)"
AVAIL="${SPIKE_DIR}/availability.json"
BENCH="${SPIKE_DIR}/bench.json"

if [ ! -f "$AVAIL" ]; then
    echo "skipped: availability.json not found — run check-availability.sh first"
    python3 << PYEOF
import json
with open("${BENCH}", 'w') as f:
    json.dump({"skipped": True, "reason": "availability.json not found"}, f, indent=2)
    f.write('\n')
PYEOF
    exit 0
fi

# Read gate status from availability.json
WEIGHTS=$(python3 -c "import json; print(json.load(open('${AVAIL}'))['weights'])")
SSM=$(python3 -c "import json; print(json.load(open('${AVAIL}'))['ssm_inference'])")

echo "Gate check: weights=${WEIGHTS}, ssm_inference=${SSM}"

# Gate: weights must be found (not gated/not_found/unknown) AND inference must be supported
if [ "$WEIGHTS" = "gated" ] || [ "$WEIGHTS" = "not_found" ] || [ "$WEIGHTS" = "unknown" ]; then
    echo "skipped: weights gate failed (status=${WEIGHTS})"
    python3 << PYEOF
import json
reason = "weights gate failed (status=${WEIGHTS})"
with open("${BENCH}", 'w') as f:
    json.dump({"skipped": True, "reason": reason}, f, indent=2)
    f.write('\n')
PYEOF
    exit 0
fi

if [ "$SSM" != "supported" ]; then
    echo "skipped: ssm_inference gate failed (status=${SSM})"
    python3 << PYEOF
import json
reason = "ssm_inference gate failed (status=${SSM}) — TENNs-LLM uses custom_code, not in llama.cpp architecture registry; state injection requires custom inference path not available on local GPU"
with open("${BENCH}", 'w') as f:
    json.dump({"skipped": True, "reason": reason}, f, indent=2)
    f.write('\n')
PYEOF
    exit 0
fi

# --- Real bench (if gates pass) ---
# Precompute hidden state for a 10K-token corpus chunk, measure state-injection
# prefill vs full-prefill on the local GPU.
echo "Gates passed — running real benchmark..."

# This path is reached only if weights are available and inference is supported.
# Currently unreachable (TENNs-LLM is custom_code, not supported in llama.cpp).
# If a future SSM model with PRECOG support lands in llama.cpp, this is where
# the bench would go: load model, precompute hidden states for a 10K-token
# corpus chunk, measure state-injection prefill vs full-prefill latency,
# output bench.json with timing data.

echo "ERROR: real bench not implemented — gates passed but no bench code path available"
python3 << PYEOF
import json
with open("${BENCH}", 'w') as f:
    json.dump({"skipped": True, "reason": "gates passed but bench implementation not yet available"}, f, indent=2)
    f.write('\n')
PYEOF
exit 0
