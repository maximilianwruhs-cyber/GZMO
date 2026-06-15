#!/usr/bin/env bash
# Smoke: skill.invoke appears in the latest pi_agent session (or SYNAPSE_TEST_SESSION_ID).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BUS="${GZMO_SYNAPSE_BUS:-$ROOT/data/Synapse/events.jsonl}"

if [[ ! -f "$BUS" ]]; then
  echo "SKIP: no synapse bus"
  exit 0
fi

SESSION_ID="${SYNAPSE_TEST_SESSION_ID:-}"
if [[ -z "$SESSION_ID" ]]; then
  while IFS= read -r line; do
    [[ -z "${line// }" ]] && continue
    if echo "$line" | grep -q '"event_type":"session_start"' && echo "$line" | grep -q '"source":"pi_agent"'; then
      SESSION_ID=$(echo "$line" | python3 -c "
import json,sys
d=json.load(sys.stdin)
print(d.get('data',{}).get('session_id',''))
" 2>/dev/null || true)
    fi
  done < "$BUS"
fi

if [[ -z "$SESSION_ID" ]]; then
  if [[ "${LIVE_SYNAPSE_TEST:-}" == "1" ]]; then
    echo "FAIL: no pi_agent session_start with session_id"
    exit 1
  fi
  echo "SKIP: no pi_agent session (run Pi with gzmo_chaos after notifier upgrade)"
  exit 0
fi

FOUND=0
while IFS= read -r line; do
  [[ -z "${line// }" ]] && continue
  echo "$line" | grep -q "\"session_id\":\"$SESSION_ID\"" || continue
  if echo "$line" | grep -q '"event_type":"skill.invoke"'; then
    FOUND=1
    break
  fi
done < "$BUS"

if [[ "$FOUND" -eq 1 ]]; then
  echo "PASS: skill.invoke found for session_id=$SESSION_ID"
  exit 0
fi

if [[ "${LIVE_SYNAPSE_TEST:-}" == "1" ]]; then
  echo "FAIL: no skill.invoke for session_id=$SESSION_ID"
  exit 1
fi
echo "SKIP: no skill.invoke for latest session (run Pi with gzmo_chaos calculate)"
exit 0
