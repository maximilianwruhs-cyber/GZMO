#!/usr/bin/env bash
# One-shot prove: doctrine row exists and living search ranks it.
# Not memory-gym: five queries, no write loop. Uses living `gzmo memory search`.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VAULT="${GZMO_LIVING_HOME:-$HOME/.gzmo-living}/data/vault.db"
GZMO="${GZMO_BIN:-$HOME/.local/bin/gzmo}"
export GZMO_CONFIG="${GZMO_CONFIG:-$HOME/.gzmo-living/gzmo.toml}"

if [[ ! -f "$VAULT" ]]; then
  echo "missing living vault: $VAULT" >&2
  exit 2
fi

n="$(sqlite3 "$VAULT" "SELECT COUNT(*) FROM honeypot WHERE is_latest=1 AND (content LIKE '%Felt Use%' OR content LIKE '%MemRL%' OR content LIKE '%utility_score%');")"
echo "doctrine_rows=$n"
if [[ "$n" -lt 1 ]]; then
  echo "FAIL: no doctrine row in latest honeypot" >&2
  exit 1
fi

search() {
  "$GZMO" memory search "$1" --limit 5 --no-scratch --json
}

rank_ok() {
  local q="$1"
  local json
  json="$(search "$q")"
  echo "---- $q ----"
  echo "$json" | python3 -c '
import json,sys
j=json.load(sys.stdin)
text=j.get("text") or ""
print(text[:800])
need=("Felt Use","MemRL","utility_score")
if not any(s in text for s in need):
    sys.exit(3)
'
}

rank_ok "Felt Use"
rank_ok "MemRL utility"
rank_ok "Felt Use Brain Feed"

abs="$(search "zzzz-nonexistent-token-9f3a2")"
echo "---- abstention ----"
echo "$abs" | python3 -c '
import json,sys
j=json.load(sys.stdin)
text=(j.get("text") or "").strip()
# honest empty or no doctrine-shaped collision claiming the bet
if "Felt Use" in text or "Glances" in text:
    sys.exit(4)
print(text[:400] or "(empty)")
'

ctrl="$(search "Prometheus PromQL")"
echo "---- control Prometheus PromQL ----"
echo "$ctrl" | python3 -c '
import json,sys
j=json.load(sys.stdin)
text=j.get("text") or ""
print(text[:400])
if "Prometheus" not in text:
    sys.exit(5)
'

echo "PASS"
