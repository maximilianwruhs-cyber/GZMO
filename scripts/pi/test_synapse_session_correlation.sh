#!/usr/bin/env bash
# Verify Pi Synapse events in a session share data.session_id.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BUS="${GZMO_SYNAPSE_BUS:-$ROOT/data/Synapse/events.jsonl}"

if [[ ! -f "$BUS" ]]; then
  echo "SKIP: no bus at $BUS (run a Pi session first)"
  exit 0
fi

# Find the most recent session_start and collect session_id
SESSION_ID=""
START_LINE=0
LINE_NUM=0
while IFS= read -r line; do
  LINE_NUM=$((LINE_NUM + 1))
  [[ -z "${line// }" ]] && continue
  if echo "$line" | grep -q '"event_type":"session_start"' && echo "$line" | grep -q '"source":"pi_agent"'; then
    SESSION_ID=$(echo "$line" | python3 -c "
import json,sys
d=json.load(sys.stdin)
print(d.get('data',{}).get('session_id',''))
" 2>/dev/null || true)
    START_LINE=$LINE_NUM
  fi
done < "$BUS"

if [[ -z "$SESSION_ID" ]]; then
  if [[ "${LIVE_SYNAPSE_TEST:-}" == "1" ]]; then
    echo "FAIL: no pi_agent session_start with session_id in bus"
    exit 1
  fi
  echo "SKIP: no pi_agent session_start with session_id in bus (upgrade synapse-notifier first)"
  exit 0
fi

echo "Checking session_id=$SESSION_ID from line $START_LINE onward..."

MISMATCH=0
CHECKED=0
TAIL=false
while IFS= read -r line; do
  [[ -z "${line// }" ]] && continue
  echo "$line" | grep -q '"source":"pi_agent"' || continue
  SID=$(echo "$line" | python3 -c "
import json,sys
d=json.load(sys.stdin)
print(d.get('data',{}).get('session_id',''))
" 2>/dev/null || true)
  if [[ -n "$SID" ]]; then
    CHECKED=$((CHECKED + 1))
    if [[ "$SID" != "$SESSION_ID" ]]; then
      echo "FAIL: mismatched session_id $SID (expected $SESSION_ID)"
      MISMATCH=$((MISMATCH + 1))
    fi
  fi
  if echo "$line" | grep -q '"event_type":"session_end"'; then
    TAIL=true
    break
  fi
done < <(tail -n +"$START_LINE" "$BUS")

if [[ "$CHECKED" -lt 1 ]]; then
  echo "SKIP: no pi_agent events with session_id after session_start"
  exit 0
fi

if [[ "$MISMATCH" -gt 0 ]]; then
  echo "FAIL: $MISMATCH session_id mismatches in $CHECKED events"
  exit 1
fi

echo "PASS: $CHECKED pi_agent events share session_id=$SESSION_ID"
exit 0
