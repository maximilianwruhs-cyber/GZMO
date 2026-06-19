#!/usr/bin/env bash
# Obolus hardware energy bridge smoke: one sample → power.jsonl row.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
BIN="${ROOT}/target/release/gzmo"
[[ -x "$BIN" ]] || BIN="${ROOT}/target/debug/gzmo"

pass() { echo "[PASS] $*"; }
fail() { echo "[FAIL] $*"; exit 1; }
warn() { echo "[WARN] $*"; }

ENERGY_EN="$(python3 -c "
import tomllib, pathlib
d = tomllib.loads(pathlib.Path('gzmo.toml').read_text())
print(d.get('obolus_analytics', {}).get('energy_sampler_enabled', False))
")"

if [[ "$ENERGY_EN" != "True" && "$ENERGY_EN" != "true" ]]; then
  warn "energy_sampler_enabled=false — enable in gzmo.toml for production sampling"
  exit 0
fi

POWER_LEDGER="$(python3 -c "
import tomllib, pathlib
d = tomllib.loads(pathlib.Path('gzmo.toml').read_text())
print(d.get('obolus_analytics', {}).get('power_ledger_path', 'data/Obolus/power.jsonl'))
")"
POWER_PATH="${ROOT}/${POWER_LEDGER}"

"$BIN" obolus sample || fail "gzmo obolus sample"

sleep 1

if [[ ! -s "$POWER_PATH" ]]; then
  fail "power.jsonl missing or empty at $POWER_PATH"
fi
pass "power.jsonl exists ($(wc -l < "$POWER_PATH" | tr -d ' ') lines)"

tail -1 "$POWER_PATH" | python3 -c "
import json, sys
row = json.loads(sys.stdin.read())
src = row.get('cpu_energy_source', '')
if src in ('rapl', 'estimate'):
    print('[PASS] cpu_energy_source=' + src)
else:
    raise SystemExit('[FAIL] missing cpu_energy_source')
"

if "$BIN" obolus balance --json >/dev/null 2>&1; then
  pass "gzmo obolus balance --json"
else
  fail "gzmo obolus balance --json"
fi

echo "OBOLUS ENERGY SMOKE: PASS"
