#!/usr/bin/env bash
# Forum Romanum — bus emit fixture + serde tests.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"
export GZMO_ROOT="$ROOT"

cargo test -p gzmo-core synapse::tests --quiet
echo "PASS: Forum Romanum serde tests"

FIXTURE=$(python3 "$ROOT/scripts/pi/emit_forum_romanum_fixture.py")
echo "$FIXTURE"
MARKER=$(echo "$FIXTURE" | sed -n 's/^MARKER=//p')
SESSION=$(echo "$FIXTURE" | sed -n 's/^SESSION_ID=//p')

BUS="${GZMO_SYNAPSE_BUS:-$ROOT/data/Synapse/events.jsonl}"
for et in agent.message proposal.created proposal.reviewed; do
  if ! grep -q "\"event_type\":\"$et\"" "$BUS" || ! tail -30 "$BUS" | grep -q "$MARKER"; then
    echo "FAIL: $et missing for marker $MARKER"
    exit 1
  fi
done

python3 <<PY
import json, sys
from pathlib import Path
bus = Path("$BUS")
marker = "$MARKER"
found = []
for line in bus.read_text().splitlines()[-40:]:
    if marker not in line:
        continue
    o = json.loads(line)
    found.append(o["event_type"])
assert "agent.message" in found
assert "proposal.created" in found
assert "proposal.reviewed" in found
# threaded chain
props = [json.loads(l) for l in bus.read_text().splitlines()[-40:] if marker in l]
for i, ev in enumerate(props[1:], start=1):
    assert ev.get("reply_to") == props[i-1]["id"], ev
print("PASS: Forum Romanum threaded fixture on bus (session=$SESSION)")
PY
