#!/usr/bin/env bash
# Live bus validation: gzmo calculate + synapse fixture + strict tests.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"
export GZMO_ROOT="$ROOT"
export GZMO_BIN="${GZMO_BIN:-$ROOT/target/release/gzmo}"
export GZMO_SYNAPSE_BUS="${GZMO_SYNAPSE_BUS:-$ROOT/data/Synapse/events.jsonl}"
export LIVE_SYNAPSE_TEST=1

echo "== 1. Daemon restart (pick up latest binary) =="
systemctl --user restart gzmo-daemon
sleep 4
systemctl --user is-active gzmo-daemon

echo ""
echo "== 2. Skill path: gzmo chaos skill calculate =="
OUT=$("$GZMO_BIN" chaos skill calculate "2+3*4" --json 2>&1 || true)
echo "$OUT" | head -5
echo "$OUT" | python3 -c "
import json,sys
raw=sys.stdin.read()
i=raw.find('{')
if i<0: raise SystemExit('no JSON in calculate output')
d=json.loads(raw[i:raw.rfind('}')+1])
assert d.get('result') in ('14', 14) or str(d.get('result'))=='14', d.get('result')
assert d.get('version')==2, d.get('version')
print('OK calculate result=14 version=2')
"

echo ""
echo "== 3. Emit live Pi synapse session (flock, notifier schema) =="
FIXTURE=$(python3 "$ROOT/scripts/pi/emit_synapse_live_fixture.py")
echo "$FIXTURE"
MARKER=$(echo "$FIXTURE" | sed -n 's/^MARKER=//p')

echo ""
echo "== 4. Verify bus contains new events =="
tail -20 "$GZMO_SYNAPSE_BUS" | grep -q "$MARKER" || { echo "FAIL: marker not on bus"; exit 1; }
tail -20 "$GZMO_SYNAPSE_BUS" | grep -q '"event_type":"skill.invoke"' || { echo "FAIL: skill.invoke missing"; exit 1; }
tail -20 "$GZMO_SYNAPSE_BUS" | grep -q '"event_type":"skill.complete"' || { echo "FAIL: skill.complete missing"; exit 1; }
echo "OK bus tail has marker + skill events"

echo ""
echo "== 5. Strict correlation + skill scripts =="
export LIVE_SYNAPSE_TEST=1
export SYNAPSE_TEST_SESSION_ID=$(echo "$FIXTURE" | sed -n 's/^SESSION_ID=//p')
bash "$ROOT/scripts/pi/test_synapse_session_correlation.sh"
bash "$ROOT/scripts/pi/test_synapse_skill_invoke.sh"

echo ""
echo "== 6. Daemon synapse pull (wait 65s for 60s poll) =="
sleep 65
journalctl --user -u gzmo-daemon --since "2 min ago" --no-pager 2>&1 | grep -i "Synapse poll" | tail -3 || true

echo ""
echo "== 7. Full infrastructure regression =="
bash "$ROOT/scripts/pi/test-infrastructure-stages.sh"

echo ""
echo "PASS: live bus validation complete"
