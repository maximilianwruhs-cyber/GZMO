#!/usr/bin/env bash
# Keep C heal: same-sitting vault → Qdrant catch-up (lab). Does not start a second overnight writer.
# Addresses F15 scar (sync cron 01:45 before distill 02:15+) for demable proof.
#
#   bash scripts/qdrant-catchup-lab.sh
# Env: QDRANT_URL (default http://127.0.0.1:6333), GZMO_VAULT, COLLECTION=honeypot
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/qdrant-catchup"
QDRANT_URL="${QDRANT_URL:-http://127.0.0.1:6333}"
COLLECTION="${QDRANT_COLLECTION:-honeypot}"
VAULT="${GZMO_VAULT:-}"
if [[ -z "$VAULT" ]]; then
  for c in "$DATA/living-appliance-home/data/vault.db" "$ROOT/data/vault.db" "$HOME/.gzmo/data/vault.db"; do
    [[ -f "$c" ]] && VAULT="$c" && break
  done
fi
mkdir -p "$OUT"
LOG="$OUT/sync.log"
: >"$LOG"

pass=0; fail=0; hold=0
declare -a ROWS=()
row() { local s="$1" n="$2" d="$3"; ROWS+=("$s|$n|$d"); case "$s" in PASS) pass=$((pass+1));; FAIL) fail=$((fail+1));; HOLD) hold=$((hold+1));; esac; echo "[$s] $n — $d"; }

echo "=== Qdrant catch-up lab (Keep C) ==="
[[ -f "$ROOT/scripts/sync-vault-to-qdrant.sh" ]] && row PASS "sync-script" "sync-vault-to-qdrant.sh" || row FAIL "sync-script" "missing"

if curl -fsS --max-time 3 "${QDRANT_URL}/" >/dev/null 2>&1; then
  row PASS "qdrant" "$QDRANT_URL"
else
  row HOLD "qdrant" "$QDRANT_URL unreachable — lab needs local/sidecar Qdrant"
fi

if [[ -n "$VAULT" && -f "$VAULT" ]]; then
  row PASS "vault" "$VAULT"
else
  row HOLD "vault" "no vault.db — set GZMO_VAULT"
fi

BEFORE=-1
AFTER=-1
if [[ "${ROWS[-1]}" != FAIL* ]] && curl -fsS --max-time 3 "${QDRANT_URL}/" >/dev/null 2>&1 && [[ -f "${VAULT:-}" ]]; then
  BEFORE="$(curl -fsS --max-time 5 "${QDRANT_URL}/collections/${COLLECTION}" 2>/dev/null | python3 -c 'import json,sys; print(json.load(sys.stdin).get("result",{}).get("points_count",-1))' || echo -1)"
  set +e
  # Prefer python sync with explicit paths when supported
  VAULT_DB="$VAULT" QDRANT_URL="$QDRANT_URL" QDRANT_COLLECTION="$COLLECTION" \
    bash "$ROOT/scripts/sync-vault-to-qdrant.sh" >>"$LOG" 2>&1
  sync_rc=$?
  set -e
  AFTER="$(curl -fsS --max-time 5 "${QDRANT_URL}/collections/${COLLECTION}" 2>/dev/null | python3 -c 'import json,sys; print(json.load(sys.stdin).get("result",{}).get("points_count",-1))' || echo -1)"
  if [[ "$sync_rc" -eq 0 ]]; then
    row PASS "sync-run" "sync exit 0 (before=$BEFORE after=$AFTER)"
  else
    row HOLD "sync-run" "sync exit $sync_rc — see $LOG (embed server may be required)"
  fi
else
  row HOLD "sync-run" "skipped — need qdrant + vault"
fi

ROWS_TSV="$(printf '%s\n' "${ROWS[@]}")"
export OUT pass fail hold ROWS_TSV BEFORE AFTER QDRANT_URL COLLECTION VAULT
python3 - <<'PY'
import json, os
from datetime import datetime, timezone
from pathlib import Path
out=Path(os.environ["OUT"]); checks={}
for line in os.environ.get("ROWS_TSV","").splitlines():
    if not line.strip(): continue
    st,n,d=line.split("|",2); checks[n]={"status":st,"detail":d}
fail_n=int(os.environ["fail"]); hold_n=int(os.environ["hold"]); pass_n=int(os.environ["pass"])
verdict="GREEN" if fail_n==0 else "RED"
advice="qdrant_catchup_ok" if fail_n==0 and checks.get("sync-run",{}).get("status")=="PASS" else (
  "qdrant_catchup_hold" if fail_n==0 else "qdrant_catchup_fail")
payload={
  "schema":"gzmo.keep.qdrant_catchup/v1",
  "generated_at":datetime.now(timezone.utc).isoformat(),
  "verdict":verdict,"ok":fail_n==0,"advice":advice,
  "counts":{"pass":pass_n,"fail":fail_n,"hold":hold_n},
  "qdrant_url":os.environ.get("QDRANT_URL"),
  "collection":os.environ.get("COLLECTION"),
  "vault":os.environ.get("VAULT"),
  "points_before":os.environ.get("BEFORE"),
  "points_after":os.environ.get("AFTER"),
  "note":"Lab catch-up — does not change CT101 cron; ADR-0003 one writer.",
  "checks":checks,
}
(out/"latest.json").write_text(json.dumps(payload,indent=2)+"\n")
print(json.dumps({"verdict":verdict,"advice":advice,"pass":pass_n,"fail":fail_n,"hold":hold_n},indent=2))
raise SystemExit(0 if fail_n==0 else 1)
PY
