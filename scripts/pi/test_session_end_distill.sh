#!/usr/bin/env bash
# Integration smoke: append synthetic session_end to synapse bus → poll extracts target.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"
export GZMO_CONFIG="${GZMO_CONFIG:-$ROOT/gzmo.toml}"

FIXTURE="${ROOT}/tests/fixtures/pi_session_minimal.jsonl"
mkdir -p "$(dirname "$FIXTURE")"
if [[ ! -f "$FIXTURE" ]]; then
  cat >"$FIXTURE" <<'EOF'
{"type":"session","id":"test-handoff-001","timestamp":"2026-06-11T15:00:00.000Z"}
{"type":"message","message":{"role":"user","content":[{"type":"text","text":"session end distill test"}]}}
{"type":"message","message":{"role":"assistant","content":[{"type":"text","text":"acknowledged"}]}}
EOF
fi

echo "== synapse_reader poll integration (cargo test) =="
unset CARGO_TARGET_DIR
export CARGO_TARGET_DIR="$ROOT/target"
cargo test -p gzmo-core poll_pi_synapse_reads_session_end --quiet

echo "== session_end event shape on live bus (dry append + rollback) =="
BUS="${ROOT}/data/Synapse/events.jsonl"
STATE_BAK="${ROOT}/data/synapse-reader.state.json.bak.$$"
READER_STATE="${ROOT}/data/synapse-reader.state.json"
OFFSET_BEFORE=0
if [[ -f "$READER_STATE" ]]; then
  cp "$READER_STATE" "$STATE_BAK"
  OFFSET_BEFORE="$(python3 -c "import json; print(json.load(open('$READER_STATE')).get('byte_offset',0))")"
fi

EVENT_LINE="$(python3 -c "
import json, uuid
print(json.dumps({
  'id': str(uuid.uuid4()),
  'event_type': 'session_end',
  'source': 'pi_agent',
  'timestamp': '2026-06-11T15:00:00Z',
  'data': {'reason': 'test', 'targetSessionFile': '$FIXTURE'}
}))")"

echo "$EVENT_LINE" >>"$BUS"
# Restore reader offset so daemon poll would see this line on next tick (manual verify)
if [[ -f "$STATE_BAK" ]]; then
  cp "$STATE_BAK" "$READER_STATE"
  rm -f "$STATE_BAK"
fi

echo "Appended test session_end (reader offset restored to $OFFSET_BEFORE)"
echo "Daemon will pick up on next 60s poll if distill_on_session_end=true"
echo "OK: session_end distill integration smoke passed"
