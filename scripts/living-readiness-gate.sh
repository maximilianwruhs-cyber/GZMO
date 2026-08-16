#!/usr/bin/env bash
# Living-stack production readiness gate (CT101 sole overnight brain).
# Exit 0 = LIVING GREEN. Laptop product GREEN is separate (product-readiness-gate.sh).
#
#   bash scripts/living-readiness-gate.sh
#   LIVING_GATE_SKIP_TAKEAWAY=1 bash scripts/living-readiness-gate.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/living-readiness"
HOST="${CT101_SSH_HOST:-ct101}"
GZMO_BIN="${CT101_GZMO_BIN:-/opt/gzmo/current/target/release/gzmo}"
MIN_FACTS="${CT101_MIN_VAULT_FACTS:-100}"
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

echo "=== Living readiness gate (CT101) ===" | tee -a "$LOG"

# 1) Dual-writer doctrine
SERVE="$(systemctl --user is-active gzmo-serve.service 2>/dev/null || true)"
SERVE="$(printf '%s\n' "$SERVE" | head -1)"
SCHED="$(systemctl --user is-active gzmo-scheduler.service 2>/dev/null || true)"
SCHED="$(printf '%s\n' "$SCHED" | head -1)"
if [[ "$SERVE" == "active" ]]; then
  row FAIL "dual-writer" "workstation gzmo-serve is active — stop it while CT101 lives"
else
  row PASS "dual-writer" "workstation serve=${SERVE:-inactive} scheduler=${SCHED:-inactive}"
fi

# 2) CT101 living smoke (daemon/sidecars/vault/health/mentor)
SMOKE_LOG="$OUT/ct101-living-smoke.log"
if bash "$ROOT/scripts/ct101-living-smoke.sh" >"$SMOKE_LOG" 2>&1; then
  facts="$(grep -E 'vault facts=' "$SMOKE_LOG" | tail -1 | sed -E 's/.*facts=([0-9]+).*/\1/' || true)"
  row PASS "ct101-living-smoke" "daemon+sidecars+vault${facts:+ ($facts facts)}+health+mentor"
else
  row FAIL "ct101-living-smoke" "see $SMOKE_LOG"
fi

# 3) Parse living health probes (required OK set)
HEALTH_LOG="$OUT/ct101-health.log"
if ssh_ct "bash -lc 'cd /opt/gzmo && GZMO_CONFIG=/opt/gzmo/gzmo.toml $GZMO_BIN health'" >"$HEALTH_LOG" 2>&1; then
  :
else
  # health may exit non-zero on WARN; still parse
  true
fi
for probe in cloud_llm prime_llm embeddings qdrant redis neo4j mcp_memory honeypot_qdrant_drift distill_queue; do
  if grep -q "\\[OK\\] ${probe}" "$HEALTH_LOG"; then
    detail="$(grep "\\[OK\\] ${probe}" "$HEALTH_LOG" | head -1 | sed 's/^[[:space:]]*//')"
    row PASS "health:${probe}" "$detail"
  elif grep -q "\\[WARN\\] ${probe}" "$HEALTH_LOG"; then
    detail="$(grep "\\[WARN\\] ${probe}" "$HEALTH_LOG" | head -1 | sed 's/^[[:space:]]*//')"
    row HOLD "health:${probe}" "$detail"
  else
    row FAIL "health:${probe}" "missing OK in CT101 health"
  fi
done
# rerank — HOLD if missing; PASS when OK
if grep -q "\\[OK\\] rerank" "$HEALTH_LOG"; then
  detail="$(grep "\\[OK\\] rerank" "$HEALTH_LOG" | head -1 | sed 's/^[[:space:]]*//')"
  row PASS "health:rerank" "${detail:-rerank OK}"
elif grep -qi "rerank.*disabled" "$HEALTH_LOG"; then
  row HOLD "health:rerank" "disabled"
else
  row HOLD "health:rerank" "not OK (non-blocking if embeddings/qdrant green)"
fi

# 4) Vault floor
facts="$(ssh_ct 'sqlite3 /opt/gzmo/data/vault.db "SELECT COUNT(*) FROM semantic_vault;"' 2>/dev/null || echo 0)"
if [[ "$facts" =~ ^[0-9]+$ ]] && (( facts >= MIN_FACTS )); then
  row PASS "vault-floor" "semantic_vault=$facts (min $MIN_FACTS)"
else
  row FAIL "vault-floor" "semantic_vault=$facts < min $MIN_FACTS"
fi

# 5) Faithfulness on living vault
bash "$ROOT/scripts/faithfulness-living.sh" >>"$LOG" 2>&1 || true
if python3 -c "import json;d=json.load(open('$DATA/faithfulness-living/latest.json')); raise SystemExit(0 if d.get('living_ok') else 1)"; then
  row PASS "faithfulness-living" "$(python3 -c "import json;d=json.load(open('$DATA/faithfulness-living/latest.json')); print(f\"{d.get('supported')}/{d.get('total')} CORE_INSIGHT claims\")")"
else
  row FAIL "faithfulness-living" "living claims not supported — see data-next/faithfulness-living/"
fi

# 6) Takeaway → distill → recall (same sitting)
if [[ "${LIVING_GATE_SKIP_TAKEAWAY:-0}" == "1" ]]; then
  row HOLD "takeaway-recall" "skipped (LIVING_GATE_SKIP_TAKEAWAY=1)"
else
  bash "$ROOT/scripts/ct101-takeaway-recall.sh" >>"$LOG" 2>&1 || true
  if python3 -c "import json;d=json.load(open('$DATA/ct101-takeaway-recall/latest.json')); raise SystemExit(0 if d.get('living_proof') else 1)"; then
    row PASS "takeaway-recall" "same-sitting living HIT"
  else
    row FAIL "takeaway-recall" "no same-sitting HIT — see data-next/ct101-takeaway-recall/"
  fi
fi

# 7) Soft CT101 probe artifact (dual-writer + smoke summary)
bash "$ROOT/scripts/ct101-living-probe.sh" >>"$LOG" 2>&1 || true
if python3 -c "import json;d=json.load(open('$DATA/ct101-living/latest.json')); raise SystemExit(0 if d.get('living_proof') else 1)"; then
  row PASS "ct101-living-probe" "$(python3 -c "import json;print(json.load(open('$DATA/ct101-living/latest.json')).get('advice',''))")"
else
  row HOLD "ct101-living-probe" "soft probe not living_proof"
fi

# 8) Workstation Prime fallback (operator cognition) — HOLD if down when CT101 prime OK
if curl -fsS --max-time 2 http://127.0.0.1:8000/v1/models >/dev/null 2>&1; then
  row PASS "workstation-prime" "http://127.0.0.1:8000 reachable (operator fallback)"
else
  if grep -q "\\[OK\\] prime_llm" "$HEALTH_LOG"; then
    row HOLD "workstation-prime" "local :8000 down; CT101 prime_llm OK via LAN"
  else
    row FAIL "workstation-prime" "no local Prime and CT101 prime_llm not OK"
  fi
fi

# 9) Goal C — in-repo living appliance compose pin (soft live probes)
bash "$ROOT/scripts/living-appliance-gate.sh" >>"$LOG" 2>&1 || true
if python3 -c "import json;d=json.load(open('$DATA/living-appliance/latest.json')); raise SystemExit(0 if d.get('ok') else 1)"; then
  row PASS "living-appliance-pin" "$(python3 -c "import json;print(json.load(open('$DATA/living-appliance/latest.json')).get('advice',''))")"
else
  row FAIL "living-appliance-pin" "compose pin invalid — see docs/LIVING_APPLIANCE.md"
fi

# 9b) Goal C — protocol smoke on CT101 (workstation Neo4j is throwaway)
bash "$ROOT/scripts/ct101-living-appliance-smoke.sh" >>"$LOG" 2>&1 || true
if [[ -f "$DATA/living-appliance-smoke/latest.json" ]] \
  && python3 -c "import json;d=json.load(open('$DATA/living-appliance-smoke/latest.json')); raise SystemExit(0 if d.get('ok') else 1)"; then
  advice="$(python3 -c "import json;print(json.load(open('$DATA/living-appliance-smoke/latest.json')).get('advice',''))")"
  if [[ "$advice" == *smoke_ok* ]]; then
    row PASS "living-appliance-smoke" "$advice"
  else
    row HOLD "living-appliance-smoke" "$advice"
  fi
else
  row FAIL "living-appliance-smoke" "CT101 protocol smoke failed — see docs/LIVING_APPLIANCE.md"
fi

# 9c) Goal C — daemon health via lab GZMO_CONFIG (never ~/.gzmo)
bash "$ROOT/scripts/living-appliance-health-smoke.sh" >>"$LOG" 2>&1 || true
if [[ -f "$DATA/living-appliance-health/latest.json" ]] \
  && python3 -c "import json;d=json.load(open('$DATA/living-appliance-health/latest.json')); raise SystemExit(0 if d.get('ok') else 1)"; then
  advice="$(python3 -c "import json;print(json.load(open('$DATA/living-appliance-health/latest.json')).get('advice',''))")"
  if [[ "$advice" == *health_ok* ]]; then
    row PASS "living-appliance-health" "$advice"
  else
    row HOLD "living-appliance-health" "$advice"
  fi
else
  row FAIL "living-appliance-health" "daemon health smoke failed — see docs/LIVING_APPLIANCE.md"
fi

# 9d) Goal C — CT101 staged pin vs live cluster shape (soft pre-promote drift)
bash "$ROOT/scripts/ct101-living-appliance-pin-check.sh" >>"$LOG" 2>&1 || true
if [[ -f "$DATA/living-appliance-pin-ct101/latest.json" ]] \
  && python3 -c "import json;d=json.load(open('$DATA/living-appliance-pin-ct101/latest.json')); raise SystemExit(0 if d.get('ok') else 1)"; then
  advice="$(python3 -c "import json;print(json.load(open('$DATA/living-appliance-pin-ct101/latest.json')).get('advice',''))")"
  if [[ "$advice" == *pin_ct101_ok* ]]; then
    row PASS "living-appliance-pin-ct101" "$advice"
  else
    row HOLD "living-appliance-pin-ct101" "$advice"
  fi
else
  row FAIL "living-appliance-pin-ct101" "staged pin check failed — ct101-sync-living-appliance.sh"
fi

# 10) Goal C — labeled gzmo-living attach (soft if not installed yet)
bash "$ROOT/scripts/living-mcp-attach-check.sh" >>"$LOG" 2>&1 || true
if python3 -c "import json;d=json.load(open('$DATA/living-mcp-attach/latest.json')); raise SystemExit(0 if d.get('ok') else 1)"; then
  advice="$(python3 -c "import json;print(json.load(open('$DATA/living-mcp-attach/latest.json')).get('advice',''))")"
  if python3 -c "import json;d=json.load(open('$DATA/living-mcp-attach/latest.json')); raise SystemExit(0 if d.get('found_living',0)>0 else 1)"; then
    row PASS "living-mcp-attach" "$advice"
  else
    row HOLD "living-mcp-attach" "$advice"
  fi
else
  row FAIL "living-mcp-attach" "living mislabeled as gzmo-memory — install-shared-mcp.sh"
fi

# 11) Unpark Wave 1.1 — herdr metabolism (soft HOLD if herdr absent)
bash "$ROOT/scripts/herdr-metabolism-check.sh" >>"$LOG" 2>&1 || true
if [[ -f "$DATA/herdr-metabolism/latest.json" ]] \
  && python3 -c "import json;d=json.load(open('$DATA/herdr-metabolism/latest.json')); raise SystemExit(0 if d.get('ok') else 1)"; then
  advice="$(python3 -c "import json;print(json.load(open('$DATA/herdr-metabolism/latest.json')).get('advice',''))")"
  if [[ "$advice" == *herdr_metabolism_ok* || "$advice" == *herdr_metabolism_living_ok* ]]; then
    row PASS "herdr-metabolism" "$advice"
  else
    row HOLD "herdr-metabolism" "$advice"
  fi
else
  row FAIL "herdr-metabolism" "herdr check failed — see docs/HERDR_METABOLISM.md"
fi

# Verdict
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
    "living_ready — CT101 metabolism production gate GREEN"
    if verdict == "GREEN"
    else "living_hold — fix FAIL rows before claiming living-stack readiness"
)
payload = {
    "schema": "gzmo.living.readiness/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "verdict": verdict,
    "ok": fail_n == 0,
    "advice": advice,
    "counts": {"pass": pass_n, "fail": fail_n, "hold": hold_n},
    "owner": {
        "living": "CT101 gzmo-daemon /opt/gzmo/",
        "lab": "workstation data-next/",
        "doc": "docs/CT101_BOUNDARY.md",
    },
    "note": "Living GREEN = sole overnight writer healthy + recall/faithfulness. Laptop product is scripts/product-readiness-gate.sh.",
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"verdict": verdict, "advice": advice, "pass": pass_n, "fail": fail_n, "hold": hold_n}, indent=2))
raise SystemExit(0 if fail_n == 0 else 1)
PY
GATE_EXIT=$?
set -e

{
  echo "# Living readiness gate"
  echo
  echo "Verdict: **$(python3 -c "import json;print(json.load(open('$OUT/latest.json'))['verdict'])")**"
  echo
  echo "| Status | Check | Detail |"
  echo "|--------|-------|--------|"
  for r in "${ROWS[@]}"; do
    IFS='|' read -r st name detail <<<"$r"
    # escape pipes in detail
    detail="${detail//|/\\|}"
    echo "| $st | $name | $detail |"
  done
  echo
  echo "See also: docs/LIVING_PRODUCTION_READINESS.md"
  echo
} >"$OUT/latest.md"

echo "=== living gate done (exit $GATE_EXIT) ===" | tee -a "$LOG"
exit "$GATE_EXIT"
