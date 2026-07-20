#!/usr/bin/env bash
# Keep quality — continuous living quality bar (USP: airgap living on one box).
# Composes living readiness + Felt Use / spark / immune / ripen / lymph / attach / airgap honesty.
#
#   bash scripts/keep-quality-gate.sh
#   LIVING_GATE_SKIP_TAKEAWAY=1 bash scripts/keep-quality-gate.sh
#   KEEP_QUALITY_SKIP_LIVING_READY=1 bash scripts/keep-quality-gate.sh   # organs only
#
# Artifact: data-next/keep-quality/latest.{json,md}
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/keep-quality"
HOST="${CT101_SSH_HOST:-ct101}"
GZMO_BIN="${CT101_GZMO_BIN:-/opt/gzmo/current/target/release/gzmo}"
DATA_DIR="${KEEP_QUALITY_DATA_DIR:-/opt/gzmo/data}"
VAULT_DB="${KEEP_QUALITY_VAULT_DB:-$DATA_DIR/vault.db}"
MIN_NONZERO_RECALL="${KEEP_QUALITY_MIN_NONZERO_RECALL:-1}"
MIN_SPARK_UNIQUE="${KEEP_QUALITY_MIN_SPARK_UNIQUE:-2}"
SPARK_LAST_N="${KEEP_QUALITY_SPARK_LAST_N:-8}"
mkdir -p "$OUT"
LOG="$OUT/gate.log"
: >"$LOG"

pass=0
fail=0
hold=0
declare -a ROWS=()

row() {
  local status="$1" name="$2" detail="$3"
  ROWS+=("$status|$name|$detail")
  case "$status" in
    PASS) pass=$((pass + 1)) ;;
    FAIL) fail=$((fail + 1)) ;;
    HOLD) hold=$((hold + 1)) ;;
  esac
  echo "[$status] $name — $detail" | tee -a "$LOG"
}

ssh_ct() {
  ssh -o ConnectTimeout=12 -o BatchMode=yes "$HOST" "$@"
}

echo "=== Keep quality gate (airgap living USP) ===" | tee -a "$LOG"

# ── 1) Living readiness (ops + faithfulness + takeaway + appliance) ──────────
if [[ "${KEEP_QUALITY_SKIP_LIVING_READY:-0}" == "1" ]]; then
  row HOLD "living-readiness" "skipped (KEEP_QUALITY_SKIP_LIVING_READY=1)"
else
  set +e
  bash "$ROOT/scripts/living-readiness-gate.sh" >>"$LOG" 2>&1
  live_rc=$?
  set -e
  if [[ -f "$DATA/living-readiness/latest.json" ]] \
    && python3 -c "import json;d=json.load(open('$DATA/living-readiness/latest.json')); raise SystemExit(0 if d.get('verdict')=='GREEN' else 1)"; then
    row PASS "living-readiness" "$(python3 -c "import json;d=json.load(open('$DATA/living-readiness/latest.json')); print(d.get('advice',''))")"
  else
    row FAIL "living-readiness" "not GREEN (exit ${live_rc:-?}) — see data-next/living-readiness/"
  fi
fi

# ── 2) Felt Use census (nonzero + depth for honest ripen) ───────────────────
bash "$ROOT/scripts/felt-use-depth.sh" >>"$LOG" 2>&1 || true
FU_JSON="${GZMO_DATA_NEXT:-$ROOT/data-next}/felt-use-depth/latest.json"
if [[ -f "$FU_JSON" ]] \
  && python3 -c "import json;d=json.load(open('$FU_JSON')); raise SystemExit(0 if d.get('ok') else 1)"; then
  fu_advice="$(python3 -c "import json;print(json.load(open('$FU_JSON')).get('advice',''))")"
  fu_depth="$(python3 -c "import json;print(json.load(open('$FU_JSON')).get('depth_ok'))")"
  fu_latest="$(python3 -c "import json;print(json.load(open('$FU_JSON')).get('census',{}).get('latest',0))")"
  fu_nz="$(python3 -c "import json;print(json.load(open('$FU_JSON')).get('census',{}).get('recall_ge1',0))")"
  if (( fu_nz >= MIN_NONZERO_RECALL )); then
    row PASS "felt-use" "latest=$fu_latest nonzero_recall=$fu_nz (min $MIN_NONZERO_RECALL)"
  else
    row FAIL "felt-use" "latest=$fu_latest nonzero_recall=$fu_nz < min $MIN_NONZERO_RECALL — living search starved"
  fi
  if [[ "$fu_depth" == "True" ]]; then
    row PASS "felt-use-depth" "$fu_advice"
  else
    row HOLD "felt-use-depth" "$fu_advice"
  fi
else
  felt_raw="$(ssh_ct "sqlite3 '$VAULT_DB' \"
SELECT
  (SELECT COUNT(*) FROM honeypot WHERE is_latest=1),
  (SELECT COUNT(*) FROM honeypot WHERE is_latest=1 AND recall_count>0);
\"" 2>/dev/null || echo "")"
  if [[ "$felt_raw" =~ ^([0-9]+)\|([0-9]+)$ ]]; then
    latest="${BASH_REMATCH[1]}"
    nonzero="${BASH_REMATCH[2]}"
    if (( nonzero >= MIN_NONZERO_RECALL )); then
      row PASS "felt-use" "latest=$latest nonzero_recall=$nonzero (min $MIN_NONZERO_RECALL)"
    else
      row FAIL "felt-use" "latest=$latest nonzero_recall=$nonzero < min $MIN_NONZERO_RECALL — living search starved"
    fi
  else
    row FAIL "felt-use" "could not query honeypot on $HOST:$VAULT_DB"
  fi
  row HOLD "felt-use-depth" "felt-use-depth.sh artifact missing — rerun scripts/felt-use-depth.sh"
fi

# ── 3) Spark refractory — unique anchors in last N ───────────────────────────
spark_json="$(ssh_ct "cat '$DATA_DIR/spark/refractory.json' 2>/dev/null" || true)"
export spark_json SPARK_LAST_N MIN_SPARK_UNIQUE
spark_eval="$(
  python3 - <<'PY'
import json, os, sys
raw = os.environ.get("spark_json") or ""
n = int(os.environ["SPARK_LAST_N"])
need = int(os.environ["MIN_SPARK_UNIQUE"])
if not raw.strip():
    print("MISSING|0|0|refractory.json missing")
    sys.exit(0)
try:
    d = json.loads(raw)
except Exception as e:
    print(f"BAD|0|0|parse:{e}")
    sys.exit(0)
# support {history:[{anchor_id|fact_id|id}]} or {recent:[...]} or list
items = d.get("history") or d.get("recent") or d.get("anchors") or d.get("entries")
if items is None and isinstance(d, list):
    items = d
if not isinstance(items, list):
    # try last_anchors / picks
    items = d.get("last_anchors") or d.get("picks") or []
ids = []
for it in items:
    if isinstance(it, str):
        ids.append(it)
    elif isinstance(it, dict):
        ids.append(str(it.get("anchor_id") or it.get("fact_id") or it.get("id") or it.get("anchor") or ""))
ids = [i for i in ids if i]
window = ids[-n:] if ids else []
uniq = len(set(window))
detail = f"last_{len(window)}_unique={uniq} need>={need}"
if not window:
    print(f"EMPTY|0|0|{detail}")
elif uniq >= need:
    print(f"PASS|{len(window)}|{uniq}|{detail}")
else:
    print(f"FAIL|{len(window)}|{uniq}|{detail} monoculture risk")
PY
)"
IFS='|' read -r spark_st spark_n spark_u spark_detail <<<"$spark_eval"
case "$spark_st" in
  PASS) row PASS "spark-refractory" "$spark_detail" ;;
  FAIL) row FAIL "spark-refractory" "$spark_detail" ;;
  EMPTY|MISSING) row HOLD "spark-refractory" "$spark_detail — run spark cycles on living host" ;;
  *) row HOLD "spark-refractory" "$spark_detail" ;;
esac

# ── 4) Immune plan artifact ──────────────────────────────────────────────────
immune_raw="$(ssh_ct "python3 - <<'PY'
import json, glob, os
base = '$DATA_DIR/immune'
latest = os.path.join(base, 'latest.json')
path = latest if os.path.isfile(latest) else None
if not path:
    plans = sorted(glob.glob(os.path.join(base, 'plan-*.json')))
    path = plans[-1] if plans else None
if not path:
    print('MISSING|0')
else:
    d = json.load(open(path))
    cands = d.get('candidates') or d.get('items') or []
    n = len(cands) if isinstance(cands, list) else int(d.get('candidate_count') or 0)
    print(f'OK|{n}|{path}')
PY" 2>/dev/null || echo "MISSING|0")"
if [[ "$immune_raw" == OK\|* ]]; then
  cand_n="$(printf '%s' "$immune_raw" | cut -d'|' -f2)"
  ipath="$(printf '%s' "$immune_raw" | cut -d'|' -f3-)"
  if [[ "$cand_n" == "0" ]]; then
    row PASS "immune" "latest plan candidates=0 ($ipath)"
  else
    row HOLD "immune" "plan candidates=$cand_n — review before apply ($ipath)"
  fi
else
  row HOLD "immune" "no immune plan yet — appears after dream consolidate"
fi

# ── 5) Ripen honesty ─────────────────────────────────────────────────────────
ripen_out="$(ssh_ct "bash -lc 'cd /opt/gzmo && GZMO_CONFIG=/opt/gzmo/gzmo.toml $GZMO_BIN ripen status'" 2>/dev/null || true)"
if printf '%s' "$ripen_out" | grep -qi "Nonzero recall_count"; then
  nz="$(printf '%s' "$ripen_out" | sed -n 's/.*Nonzero recall_count:\*\* *\([0-9][0-9]*\).*/\1/p' | head -1)"
  [[ -z "$nz" ]] && nz="$(printf '%s' "$ripen_out" | sed -n 's/.*Nonzero recall_count: *\([0-9][0-9]*\).*/\1/p' | head -1)"
  core="$(printf '%s' "$ripen_out" | sed -n 's/.*knowledge_core.db rows:\*\* *\([0-9][0-9]*\).*/\1/p' | head -1)"
  [[ -z "$core" ]] && core="$(printf '%s' "$ripen_out" | sed -n 's/.*knowledge_core.db rows: *\([0-9][0-9]*\).*/\1/p' | head -1)"
  if printf '%s' "$ripen_out" | grep -qi "Starved"; then
    row HOLD "ripen" "starved_recall (honest) nonzero=${nz:-?} core=${core:-?}"
  elif [[ -n "$nz" && "$nz" != "0" ]]; then
    row PASS "ripen" "nonzero_recall=$nz core_rows=${core:-?} — honest status"
  else
    row HOLD "ripen" "status parsed but recall unclear — see gate.log"
  fi
  printf '%s\n' "$ripen_out" >>"$LOG"
else
  row FAIL "ripen" "gzmo ripen status failed on $HOST"
fi

# ── 6) Night lymph ───────────────────────────────────────────────────────────
lymph_raw="$(ssh_ct "test -f '$DATA_DIR/night-lymph/latest.json' && python3 -c \"
import json
d=json.load(open('$DATA_DIR/night-lymph/latest.json'))
nid = d.get('night_id') or d.get('date') or 'present'
sp = d.get('sparks')
n = len(sp) if isinstance(sp, list) else sp
print(f'OK night_id={nid} sparks={n}')
\" || echo MISSING" 2>/dev/null || echo MISSING)"
if [[ "$lymph_raw" == OK* ]]; then
  row PASS "night-lymph" "$lymph_raw"
else
  row HOLD "night-lymph" "latest.json missing — after overnight dream/spark"
fi

# ── 7) Living MCP attach (local label) ───────────────────────────────────────
bash "$ROOT/scripts/living-mcp-attach-check.sh" >>"$LOG" 2>&1 || true
if [[ -f "$DATA/living-mcp-attach/latest.json" ]] \
  && python3 -c "import json;d=json.load(open('$DATA/living-mcp-attach/latest.json')); raise SystemExit(0 if d.get('ok') else 1)"; then
  advice="$(python3 -c "import json;print(json.load(open('$DATA/living-mcp-attach/latest.json')).get('advice',''))")"
  if python3 -c "import json;d=json.load(open('$DATA/living-mcp-attach/latest.json')); raise SystemExit(0 if d.get('found_living',0)>0 else 1)"; then
    row PASS "mcp-attach" "$advice"
  else
    row HOLD "mcp-attach" "$advice — wire local gzmo-living on the living box"
  fi
else
  row FAIL "mcp-attach" "living mislabeled as gzmo-memory — see docs/MCP_LOCAL_ATTACH.md"
fi

# ── 8) Airgap honesty (local engines preferred; cloud not required) ──────────
HEALTH_LOG="$OUT/airgap-health.log"
if ssh_ct "bash -lc 'cd /opt/gzmo && GZMO_CONFIG=/opt/gzmo/gzmo.toml $GZMO_BIN health'" >"$HEALTH_LOG" 2>&1; then
  :
fi
prime_ok=0
cloud_ok=0
grep -q '\[OK\] prime_llm' "$HEALTH_LOG" && prime_ok=1
grep -q '\[OK\] cloud_llm' "$HEALTH_LOG" && cloud_ok=1
embed_ok=0
grep -q '\[OK\] embeddings' "$HEALTH_LOG" && embed_ok=1
if (( prime_ok == 1 && embed_ok == 1 )); then
  row PASS "airgap-honesty" "prime_llm+embeddings OK on living host (cloud optional)"
elif (( prime_ok == 1 )); then
  row HOLD "airgap-honesty" "prime OK; embeddings not OK — vector path degraded"
elif (( cloud_ok == 1 && prime_ok == 0 )); then
  row FAIL "airgap-honesty" "cloud_llm OK but prime_llm not — core path not airgap-capable"
else
  row FAIL "airgap-honesty" "no local prime_llm — see $HEALTH_LOG"
fi

# ── Verdict ──────────────────────────────────────────────────────────────────
export OUT pass fail hold
set +e
python3 - <<'PY'
import json, os
from datetime import datetime, timezone
from pathlib import Path

out = Path(os.environ["OUT"])
pass_n = int(os.environ["pass"])
fail_n = int(os.environ["fail"])
hold_n = int(os.environ["hold"])
verdict = "GREEN" if fail_n == 0 else "RED"
advice = (
    "keep_quality_ready — living quality bar GREEN (USP airgap living)"
    if verdict == "GREEN"
    else "keep_quality_hold — fix FAIL rows before claiming Keep quality"
)
payload = {
    "schema": "gzmo.keep.quality/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "verdict": verdict,
    "ok": fail_n == 0,
    "advice": advice,
    "counts": {"pass": pass_n, "fail": fail_n, "hold": hold_n},
    "usp": "full living on one airgapped box",
    "doc": "docs/KEEP_QUALITY.md",
    "doctrine": "docs/ADR-0004-airgap-living-usp.md",
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"verdict": verdict, "advice": advice, "pass": pass_n, "fail": fail_n, "hold": hold_n}, indent=2))
raise SystemExit(0 if fail_n == 0 else 1)
PY
GATE_EXIT=$?
set -e

{
  echo "# Keep quality gate"
  echo
  echo "Verdict: **$(python3 -c "import json;print(json.load(open('$OUT/latest.json'))['verdict'])")**"
  echo
  echo "| Status | Check | Detail |"
  echo "|--------|-------|--------|"
  for r in "${ROWS[@]}"; do
    IFS='|' read -r st name detail <<<"$r"
    detail="${detail//|/\\|}"
    echo "| $st | $name | $detail |"
  done
  echo
  echo "See: docs/KEEP_QUALITY.md · docs/ADR-0004-airgap-living-usp.md"
  echo
} >"$OUT/latest.md"

echo "=== keep-quality done (exit $GATE_EXIT) ===" | tee -a "$LOG"
exit "$GATE_EXIT"
