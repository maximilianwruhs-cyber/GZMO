#!/usr/bin/env bash
# Brain Feed gate — satellites that nourish the living vault (P0 nutrients).
#   bash scripts/brain-feed-check.sh
# Artifact: data-next/brain-feed/latest.{json,md}
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/brain-feed"
HOST="${CT101_SSH_HOST:-ct101}"
VAULT_DB="${KEEP_QUALITY_VAULT_DB:-/opt/gzmo/data/vault.db}"
MIN_NONZERO_RECALL="${KEEP_QUALITY_MIN_NONZERO_RECALL:-1}"
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

echo "=== Brain Feed check ===" | tee -a "$LOG"

# Dual-writer guard
SERVE="$(systemctl --user is-active gzmo-serve.service 2>/dev/null || true)"
SERVE="$(printf '%s\n' "$SERVE" | head -1)"
if [[ "$SERVE" == "active" ]]; then
  row FAIL "dual-writer" "workstation gzmo-serve active — stop before feeding living vault"
else
  row PASS "dual-writer" "serve=${SERVE:-inactive}"
fi

# herdr / takeaway surface
bash "$ROOT/scripts/herdr-metabolism-check.sh" >>"$LOG" 2>&1 || true
if [[ -f "$DATA/herdr-metabolism/latest.json" ]] \
  && python3 -c "import json;d=json.load(open('$DATA/herdr-metabolism/latest.json')); raise SystemExit(0 if d.get('ok') else 1)"; then
  advice="$(python3 -c "import json;print(json.load(open('$DATA/herdr-metabolism/latest.json')).get('advice',''))")"
  row PASS "herdr-takeaway" "$advice"
else
  row HOLD "herdr-takeaway" "herdr check soft — plugin optional; see HERDR_METABOLISM.md"
fi

# Living takeaway→recall artifact (prefer recent living_proof)
if [[ -f "$DATA/ct101-takeaway-recall/latest.json" ]] \
  && python3 -c "import json;d=json.load(open('$DATA/ct101-takeaway-recall/latest.json')); raise SystemExit(0 if d.get('living_proof') else 1)"; then
  row PASS "takeaway-recall" "living_proof HIT present"
else
  row HOLD "takeaway-recall" "no living_proof yet — bash scripts/ct101-takeaway-recall.sh"
fi

# Takeaway as side-effect (remind surfaces — never memory-gym)
bash "$ROOT/scripts/takeaway-side-effect-remind.sh" >>"$LOG" 2>&1 || true
if [[ -f "$DATA/takeaway-side-effect/latest.json" ]] \
  && python3 -c "import json;d=json.load(open('$DATA/takeaway-side-effect/latest.json')); raise SystemExit(0 if d.get('ok') else 1)"; then
  row PASS "takeaway-side-effect" "$(python3 -c "import json;print(json.load(open('$DATA/takeaway-side-effect/latest.json')).get('advice',''))")"
else
  row FAIL "takeaway-side-effect" "remind surfaces missing — scripts/takeaway-side-effect-remind.sh"
fi

# tinyFolder living enqueue (idempotent warm-up: only stage a demo if no fresh drop in 12h)
NEWEST_DROP="$(ls -t "$DATA/inbox"/drop-*.md "$DATA/inbox"/processed/*.md "$DATA/inbox"/processed/*/*.md 2>/dev/null | head -1 || true)"
if [[ -z "$NEWEST_DROP" ]] || (( $(date +%s) - $(stat -c %Y "$NEWEST_DROP") > 43200 )); then
  bash "$ROOT/scripts/tinyfolder-drop.sh" --demo --living >>"$LOG" 2>&1 || true
fi
bash "$ROOT/scripts/tinyfolder-check.sh" >>"$LOG" 2>&1 || true
if [[ -f "$DATA/tinyfolder/living-enqueue.json" ]] \
  && python3 -c "import json;d=json.load(open('$DATA/tinyfolder/living-enqueue.json')); raise SystemExit(0 if d.get('ok') else 1)"; then
  row PASS "tinyfolder-living" "$(python3 -c "import json;print(json.load(open('$DATA/tinyfolder/living-enqueue.json')).get('advice',''))")"
else
  row FAIL "tinyfolder-living" "living-enqueue missing/not ok"
fi

# Felt Use census on living vault (nonzero + depth for ripen honesty)
bash "$ROOT/scripts/felt-use-depth.sh" >>"$LOG" 2>&1 || true
if [[ -f "$DATA/felt-use-depth/latest.json" ]] \
  && python3 -c "import json;d=json.load(open('$DATA/felt-use-depth/latest.json')); raise SystemExit(0 if d.get('ok') else 1)"; then
  advice="$(python3 -c "import json;print(json.load(open('$DATA/felt-use-depth/latest.json')).get('advice',''))")"
  depth_ok="$(python3 -c "import json;print(json.load(open('$DATA/felt-use-depth/latest.json')).get('depth_ok'))")"
  if [[ "$depth_ok" == "True" ]]; then
    row PASS "felt-use" "$advice"
    row PASS "felt-use-depth" "$advice"
  else
    # Nonzero still required for Brain Feed P0; depth thin is HOLD not RED
    felt_raw="$(ssh -o ConnectTimeout=12 -o BatchMode=yes "$HOST" "sqlite3 '$VAULT_DB' \"
SELECT
  (SELECT COUNT(*) FROM honeypot WHERE is_latest=1),
  (SELECT COUNT(*) FROM honeypot WHERE is_latest=1 AND recall_count>0);
\"" 2>/dev/null || echo "")"
    if [[ "$felt_raw" =~ ^([0-9]+)\|([0-9]+)$ ]]; then
      latest="${BASH_REMATCH[1]}"
      nonzero="${BASH_REMATCH[2]}"
      if (( nonzero >= MIN_NONZERO_RECALL )); then
        row PASS "felt-use" "latest=$latest nonzero_recall=$nonzero"
      else
        row FAIL "felt-use" "nonzero_recall=$nonzero < min $MIN_NONZERO_RECALL"
      fi
    else
      row FAIL "felt-use" "could not query living honeypot"
    fi
    row HOLD "felt-use-depth" "$advice"
  fi
else
  row FAIL "felt-use" "felt-use-depth census failed — scripts/felt-use-depth.sh"
  row FAIL "felt-use-depth" "unreachable or not ok"
fi

# Serendipity cadence (digest + promote dry-run + checklist artifact)
bash "$ROOT/scripts/serendipity-cadence.sh" >>"$LOG" 2>&1 || true
if [[ -f "$DATA/serendipity/cadence-latest.json" ]] \
  && python3 -c "import json;d=json.load(open('$DATA/serendipity/cadence-latest.json')); raise SystemExit(0 if d.get('ok') else 1)"; then
  row PASS "serendipity-cadence" "$(python3 -c "import json;print(json.load(open('$DATA/serendipity/cadence-latest.json')).get('advice',''))")"
else
  row FAIL "serendipity-cadence" "cadence failed — scripts/serendipity-cadence.sh"
fi
if [[ -f "$DATA/serendipity/promote-latest.json" ]] \
  && python3 -c "
import json
d=json.load(open('$DATA/serendipity/promote-latest.json'))
ok=bool(d.get('ok')) and (bool(d.get('dry_run')) or bool(d.get('applied')))
raise SystemExit(0 if ok else 1)
"; then
  n="$(python3 -c "import json;print(json.load(open('$DATA/serendipity/promote-latest.json')).get('candidate_count',0))")"
  dry="$(python3 -c "import json;print(json.load(open('$DATA/serendipity/promote-latest.json')).get('dry_run'))")"
  applied_n="$(python3 -c "import json;print(len(json.load(open('$DATA/serendipity/promote-latest.json')).get('applied') or []))")"
  if [[ "$dry" == "True" ]]; then
    if [[ "$n" == "0" ]]; then
      row HOLD "serendipity-promote" "dry-run ok but 0 candidates — run spark on living host"
    else
      row PASS "serendipity-promote" "dry-run ok candidates=$n"
    fi
  else
    row PASS "serendipity-promote" "applied=$applied_n candidates=$n (human-gated; auto_apply=false)"
  fi
else
  row FAIL "serendipity-promote" "promote dry-run/apply failed — see data-next/serendipity/"
fi

# Apply proof / recent human apply (closes 0-apply remind without auto-apply)
if [[ -f "$DATA/serendipity/apply-proof-latest.json" ]] \
  && python3 -c "import json;d=json.load(open('$DATA/serendipity/apply-proof-latest.json')); raise SystemExit(0 if d.get('ok') and d.get('apply_ok') else 1)"; then
  row PASS "serendipity-apply-proof" "$(python3 -c "import json;print(json.load(open('$DATA/serendipity/apply-proof-latest.json')).get('advice',''))")"
elif [[ -f "$DATA/serendipity/cadence-latest.json" ]] \
  && python3 -c "
import json
d=json.load(open('$DATA/serendipity/cadence-latest.json'))
ok=int(d.get('applied_total') or 0)>0 and not d.get('stale_apply', True)
raise SystemExit(0 if ok else 1)
"; then
  row PASS "serendipity-apply-proof" "$(python3 -c "import json;d=json.load(open('$DATA/serendipity/cadence-latest.json')); print(f\\\"applies_logged={d.get('applied_total')} last={d.get('last_apply_at')}\\\")")"
else
  row HOLD "serendipity-apply-proof" "no recent apply — bash scripts/serendipity-apply-proof.sh --apply"
fi

# Dream compact lab presence (hygiene — soft)
if [[ -x "$ROOT/scripts/dream-compact-lab.sh" ]]; then
  row PASS "dream-compact" "dream-compact-lab.sh present (soft; off GREEN math)"
else
  row HOLD "dream-compact" "dream-compact-lab.sh missing"
fi

# Docs
if [[ -f "$ROOT/docs/BRAIN_FEED.md" ]]; then
  row PASS "doctrine" "docs/BRAIN_FEED.md"
else
  row FAIL "doctrine" "missing BRAIN_FEED.md"
fi

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
    "brain_feed_ready — nutrient loops demable toward living vault"
    if verdict == "GREEN"
    else "brain_feed_hold — fix FAIL rows (see docs/BRAIN_FEED.md)"
)
payload = {
    "schema": "gzmo.brain_feed.check/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "verdict": verdict,
    "ok": fail_n == 0,
    "advice": advice,
    "counts": {"pass": pass_n, "fail": fail_n, "hold": hold_n},
    "doc": "docs/BRAIN_FEED.md",
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"verdict": verdict, "advice": advice, "pass": pass_n, "fail": fail_n, "hold": hold_n}, indent=2))
raise SystemExit(0 if fail_n == 0 else 1)
PY
GATE_EXIT=$?
set -e

{
  echo "# Brain Feed check"
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
  echo "See: docs/BRAIN_FEED.md"
  echo
} >"$OUT/latest.md"

echo "=== brain-feed done (exit $GATE_EXIT) ===" | tee -a "$LOG"
exit "$GATE_EXIT"
