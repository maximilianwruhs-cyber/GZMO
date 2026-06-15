#!/usr/bin/env bash
# Synapse Writer gate — unit tests + headless CLI bypass smoke.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"
GZMO="${GZMO_BIN:-$ROOT/target/release/gzmo}"

cargo test -p gzmo-core synapse_writer --quiet
echo "PASS: synapse_writer unit tests"

export GZMO_SYNAPSE_GATE_BYPASS=1
OUT=$("$GZMO" chaos skill calculate "2+3*4" --json 2>&1)
echo "$OUT" | python3 -c "
import json,sys
raw=sys.stdin.read()
i=raw.find('{')
d=json.loads(raw[i:raw.rfind('}')+1])
assert str(d.get('result')) in ('14', '14.0') or d.get('result')==14
print('PASS: gate bypass CLI calculate')
"

echo "PASS: synapse writer gate tests"
