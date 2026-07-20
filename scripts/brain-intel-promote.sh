#!/usr/bin/env bash
# Brain Feed P1 — intelligence promote ritual (suggestions only, never auto-apply).
# Collects calibration fuse sibling + Arena champion-suggestion; verifies daemon
# jobs are untouched. Human merges into living toml on the living host.
#
#   bash scripts/brain-intel-promote.sh
# Artifact: data-next/brain-intel/latest.{json,md}
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/brain-intel"
HOST="${CT101_SSH_HOST:-ct101}"
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

echo "=== Brain intel promote (human only) ===" | tee -a "$LOG"

# Never auto-apply
row PASS "no-auto-apply" "this script never writes /opt/gzmo/gzmo.toml or starts daemon"

# Dual-writer
SERVE="$(systemctl --user is-active gzmo-serve.service 2>/dev/null || true)"
SERVE="$(printf '%s\n' "$SERVE" | head -1)"
if [[ "$SERVE" == "active" ]]; then
  row FAIL "dual-writer" "gzmo-serve active — refuse intel promote ritual"
else
  row PASS "dual-writer" "serve=${SERVE:-inactive}"
fi

# Calibration / fused suggestion siblings (lab)
FUSED_CANDIDATES=(
  "$ROOT/config/gzmo-next-fused.toml"
  "$DATA/fused/gzmo-fused.toml"
  "$ROOT/config/gzmo-fused.toml"
)
fused=""
for f in "${FUSED_CANDIDATES[@]}"; do
  if [[ -f "$f" ]]; then fused="$f"; break; fi
done
if [[ -n "$fused" ]]; then
  row PASS "calibration-suggestion" "fused sibling present: $fused (human merge only)"
else
  row HOLD "calibration-suggestion" "no fused toml yet — run bench-to-fuse / calibrate theatre"
fi

# Write living pin ritual stub (never applies)
SUGGEST="$OUT/living-pin-suggestion.md"
{
  echo "# Living engine pin suggestion (human only)"
  echo
  echo "Generated: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo
  echo "## Calibration"
  if [[ -n "$fused" ]]; then
    echo "- Review: \`$fused\`"
    echo "- Diff against living: copy sections into living \`gzmo.toml\` by hand (or \`gzmo config promote-fused --diff\` on lab next only)."
    echo "- **Never** silent overwrite of \`/opt/gzmo/gzmo.toml\` from this workstation."
  else
    echo "- No fused sibling found. Run calibration on a machine with the candidate local model."
  fi
  echo
  echo "## Arena champion"
  echo "- Expect: \`data-next/arena/champion-suggestion.toml\` (sibling only)."
  echo "- Human promotes champion into living engine config after review."
  echo
  echo "## IpW / Forge (P1b)"
  echo "- After Arena suggestion is boring: \`ipw-route-demo.sh\` / \`forge-lab-demo.sh\`."
  echo "- Still human promote only; \`daemon_jobs_touched=false\`."
  echo
} >"$SUGGEST"
row PASS "living-pin-doc" "$SUGGEST"

# Arena champion suggestion
CHAMP="$DATA/arena/champion-suggestion.toml"
if [[ -f "$CHAMP" ]]; then
  row PASS "arena-champion" "champion-suggestion.toml present (human promote only)"
else
  # Soft: run arena-lab demo if available to refresh suggestion shape
  if [[ -x "$ROOT/scripts/arena-lab-demo.sh" ]]; then
    bash "$ROOT/scripts/arena-lab-demo.sh" >>"$LOG" 2>&1 || true
  fi
  if [[ -f "$CHAMP" ]]; then
    row PASS "arena-champion" "champion-suggestion.toml from arena-lab-demo"
  else
    row HOLD "arena-champion" "no champion-suggestion.toml — run arena-night / arena-lab-demo"
  fi
fi

# Daemon jobs untouched on living host (soft SSH)
DAEMON_TOUCHED=0
if ssh -o ConnectTimeout=12 -o BatchMode=yes "$HOST" \
  "test -f /opt/gzmo/data/arena/daemon_jobs_touched && cat /opt/gzmo/data/arena/daemon_jobs_touched" 2>/dev/null \
  | grep -qi true; then
  DAEMON_TOUCHED=1
fi
# Also check lab arena latest.json
if [[ -f "$DATA/arena/latest.json" ]] \
  && python3 -c "import json;d=json.load(open('$DATA/arena/latest.json')); raise SystemExit(0 if d.get('daemon_jobs_touched') is True else 1)" 2>/dev/null; then
  DAEMON_TOUCHED=1
fi
if (( DAEMON_TOUCHED == 1 )); then
  row FAIL "daemon-jobs" "arena claims daemon_jobs_touched — Brain Feed forbids auto job mutation"
else
  row PASS "daemon-jobs" "no daemon job mutation claimed"
fi

# promote-fused merge helper present (lab)
if [[ -f "$ROOT/scripts/promote-fused-merge.py" ]]; then
  row PASS "promote-fused-tool" "scripts/promote-fused-merge.py (lab/next — not CT101 auto)"
else
  row HOLD "promote-fused-tool" "promote-fused-merge.py missing"
fi

export OUT pass fail hold fused SUGGEST CHAMP
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
    "brain_intel_suggest_ready — human merge only; no auto-apply"
    if verdict == "GREEN"
    else "brain_intel_hold — fix FAIL rows"
)
payload = {
    "schema": "gzmo.brain_feed.intel_promote/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "verdict": verdict,
    "ok": fail_n == 0,
    "advice": advice,
    "auto_apply": False,
    "fused_suggestion": os.environ.get("fused") or None,
    "living_pin_doc": os.environ.get("SUGGEST"),
    "arena_champion": os.environ.get("CHAMP") if Path(os.environ.get("CHAMP") or "").is_file() else None,
    "counts": {"pass": pass_n, "fail": fail_n, "hold": hold_n},
    "doc": "docs/BRAIN_FEED.md",
    "operator": [
        "Review data-next/brain-intel/living-pin-suggestion.md",
        "On living host: merge calibration/champion by hand into gzmo.toml",
        "Never run promote-fused --apply against CT101 from this script",
    ],
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
md = [
    "# Brain intel promote",
    "",
    f"Verdict: **{verdict}**",
    "",
    f"- Advice: {advice}",
    f"- Auto-apply: **false**",
    f"- Pin doc: `{payload['living_pin_doc']}`",
    "",
    "See docs/BRAIN_FEED.md",
    "",
]
(out / "latest.md").write_text("\n".join(md) + "\n", encoding="utf-8")
print(json.dumps({"verdict": verdict, "advice": advice, "pass": pass_n, "fail": fail_n, "hold": hold_n}, indent=2))
raise SystemExit(0 if fail_n == 0 else 1)
PY
GATE_EXIT=$?
set -e

echo "=== brain-intel done (exit $GATE_EXIT) ===" | tee -a "$LOG"
echo "Pin ritual: $SUGGEST" | tee -a "$LOG"
exit "$GATE_EXIT"
