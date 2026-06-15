#!/usr/bin/env bash
# Tier 3 live chat stress — gzmo-chaos QA
# See docs/GZMO_CHAOS_AGENT_GUIDE.md §13
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

pkill -f './target/release/gzmo daemon' || true
rm -f /tmp/gzmo_rust.pid 2>/dev/null || true

# Shell skills use _llm_helper.sh — need FULL chat/completions URL, not gzmo.toml base /v1
export GZMO_LLM_URL="${GZMO_LLM_URL:-http://localhost:8000/v1/chat/completions}"
export GZMO_LLM_MODEL="${GZMO_LLM_MODEL:-gemma-4-26b-a4b-it}"

unset CARGO_TARGET_DIR
cargo build --release -p gzmo-cli -q

LOG="${TIER3_LOG:-/tmp/tier3_stress_chat.log}"
echo "Tier 3 stress → $LOG"
echo "GZMO_LLM_URL=$GZMO_LLM_URL GZMO_LLM_MODEL=$GZMO_LLM_MODEL"

{
  sleep 2;  printf '%s\n' '/chaos'
  sleep 5;  printf '%s\n' '/story'; sleep 35; printf '%s\n' '/chaos'
  sleep 5;  printf '%s\n' '/story'; sleep 35; printf '%s\n' '/chaos'
  sleep 5;  printf '%s\n' '/story'; sleep 35; printf '%s\n' '/chaos'
  sleep 5;  printf '%s\n' '/joke';  sleep 20; printf '%s\n' '/chaos'
  sleep 3;  printf '%s\n' '/stabilize'; sleep 3; printf '%s\n' '/chaos'
  sleep 2;  printf '%s\n' '/quit'
} | ./target/release/gzmo chat 2>&1 | tee "$LOG"

echo "Done. Grep ρ mod lines:"
grep -E 'Lorenz ρ mod|ρ_eff|stabilized|crystallized|Chaos engine running' "$LOG" || true
