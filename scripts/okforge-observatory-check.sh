#!/usr/bin/env bash
# OKForge + Observatory knowledge-plane production gate (workstation).
# Never starts gzmo-serve. Not on living GREEN math.
#
#   bash scripts/okforge-observatory-check.sh
#   bash scripts/okforge-observatory-check.sh --docs-only
#   OKFORGE_CHECK_SOFT=1 bash scripts/okforge-observatory-check.sh   # HTTP WARN if forge down
#
# Artifact: data-next/okforge-observatory/latest.{json,md}
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/okforge-observatory"
OKFORGE_URL="${OKFORGE_PROBE_URL:-http://127.0.0.1:3000/observatory}"
TOKEN_ENV="${OKFORGE_TOKEN_ENV:-OKFORGE_TOKEN}"
STALE_SECS="${OKFORGE_WIKI_STALE_SECS:-93600}" # 26h
DOCS_ONLY=0
[[ "${1:-}" == "--docs-only" ]] && DOCS_ONLY=1
mkdir -p "$OUT"

pass=0
fail=0
warn=0
hold=0
declare -a ROWS=()

row() {
  local status="$1" name="$2" detail="$3"
  ROWS+=("$status|$name|$detail")
  case "$status" in
    PASS) pass=$((pass + 1)) ;;
    FAIL) fail=$((fail + 1)) ;;
    WARN) warn=$((warn + 1)) ;;
    HOLD) hold=$((hold + 1)) ;;
  esac
  echo "[$status] $name — $detail"
}

echo "=== OKForge / Observatory production check ==="

[[ -f "$ROOT/docs/OKFORGE_PRODUCTION.md" ]] && row PASS "prod-doc" "OKFORGE_PRODUCTION.md" || row FAIL "prod-doc" "missing"
[[ -x "$ROOT/scripts/wiki-okforge-living-push.sh" ]] && row PASS "living-push" "wiki-okforge-living-push.sh" || row FAIL "living-push" "missing"
[[ -x "$ROOT/scripts/wiki-push-gated.sh" ]] && row PASS "gated-push" "wiki-push-gated.sh" || row HOLD "gated-push" "missing"

if rg -n 'okforge-observatory-check' "$ROOT/scripts/living-readiness-gate.sh" >/dev/null 2>&1 \
  || rg -n 'okforge-observatory-check' "$ROOT/scripts/keep-quality-gate.sh" >/dev/null 2>&1; then
  row FAIL "not-living-required" "knowledge-plane check wired into living/keep-quality GREEN"
else
  row PASS "not-living-required" "living GREEN independent of OKForge glass"
fi

if rg -n ':7777' "$ROOT/docs/ct101-systems/110-external-nodes/observatory.md" >/dev/null 2>&1 \
  && ! rg -n -i 'retired' "$ROOT/docs/ct101-systems/110-external-nodes/observatory.md" >/dev/null 2>&1; then
  row FAIL "docs-observatory" "observatory.md still treats :7777 as live"
else
  row PASS "docs-observatory" "observatory.md names current :3000 / TUI surfaces"
fi

serve="$(systemctl --user is-active gzmo-serve.service 2>/dev/null || true)"
serve="$(printf '%s\n' "$serve" | head -1)"
if [[ "$serve" == "active" ]]; then
  row FAIL "dual-writer" "workstation gzmo-serve active — stop before knowledge-plane ops"
else
  row PASS "dual-writer" "serve=${serve:-inactive}"
fi

if [[ "$DOCS_ONLY" == "1" ]]; then
  row HOLD "okforge-http" "skipped (--docs-only)"
  row HOLD "token-env" "skipped (--docs-only)"
  row HOLD "wiki-meta" "skipped (--docs-only)"
else
  code="$(curl -sS -o /dev/null -w '%{http_code}' --max-time 5 "$OKFORGE_URL" || true)"
  if [[ "$code" =~ ^[1-4][0-9][0-9]$ ]]; then
    row PASS "okforge-http" "HTTP $code $OKFORGE_URL"
  elif [[ "${OKFORGE_CHECK_SOFT:-0}" == "1" ]]; then
    row WARN "okforge-http" "unreachable $OKFORGE_URL (code=${code:-none}, soft)"
  else
    row FAIL "okforge-http" "unreachable $OKFORGE_URL (code=${code:-none})"
  fi

  if [[ -n "${!TOKEN_ENV:-}" ]]; then
    row PASS "token-env" "$TOKEN_ENV is set (value not printed)"
  else
    row FAIL "token-env" "$TOKEN_ENV unset — wiki push cannot run"
  fi

  meta=""
  for cand in \
    "$DATA/wiki-push-latest.json" \
    "$ROOT/data-next/wiki-push-latest.json" \
    "$ROOT/gzmo-core/data/wiki-push-latest.json" \
    "$ROOT/data/wiki-push-latest.json"; do
    [[ -f "$cand" ]] && meta="$cand" && break
  done
  if [[ -z "$meta" ]]; then
    row WARN "wiki-meta" "no wiki-push-latest.json yet — run living satellite or gzmo wiki push"
  else
    python3 - "$meta" "$STALE_SECS" <<'PY' && rc=0 || rc=$?
import json, sys, datetime
from pathlib import Path
path = Path(sys.argv[1])
stale_secs = int(sys.argv[2])
v = json.loads(path.read_text())
healthy = v.get("healthy")
ts = v.get("timestamp") or ""
age = None
if ts:
    try:
        dt = datetime.datetime.fromisoformat(ts.replace("Z", "+00:00"))
        age = (datetime.datetime.now(datetime.timezone.utc) - dt).total_seconds()
    except Exception:
        age = None
if healthy is False:
    raise SystemExit(2)
if age is not None and age > stale_secs:
    raise SystemExit(3)
raise SystemExit(0)
PY
    case "$rc" in
      0) row PASS "wiki-meta" "$meta healthy" ;;
      2) row FAIL "wiki-meta" "$meta healthy=false — see error/skipped_reason" ;;
      3) row WARN "wiki-meta" "$meta older than ${STALE_SECS}s" ;;
      *) row WARN "wiki-meta" "$meta unreadable" ;;
    esac
  fi
fi

verdict=GREEN
[[ "$fail" -eq 0 ]] || verdict=RED
[[ "$fail" -eq 0 && ( "$warn" -gt 0 || "$hold" -gt 0 ) ]] && verdict=YELLOW

generated="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
{
  echo "# okforge-observatory — $verdict"
  echo
  echo "generated_at: $generated"
  echo
  for r in "${ROWS[@]}"; do
    IFS='|' read -r st name detail <<<"$r"
    echo "- [$st] $name — $detail"
  done
} >"$OUT/latest.md"

python3 - "$OUT" "$verdict" "$pass" "$fail" "$warn" "$hold" "$generated" "${ROWS[@]}" <<'PY'
import json, sys
from pathlib import Path
out = Path(sys.argv[1])
verdict, pass_n, fail_n, warn_n, hold_n, generated = sys.argv[2:8]
rows = []
for raw in sys.argv[8:]:
    st, name, detail = raw.split("|", 2)
    rows.append({"status": st, "name": name, "detail": detail})
payload = {
    "schema": "gzmo.okforge_observatory/v1",
    "generated_at": generated,
    "verdict": verdict,
    "pass": int(pass_n),
    "fail": int(fail_n),
    "warn": int(warn_n),
    "hold": int(hold_n),
    "ok": verdict != "RED",
    "rows": rows,
    "note": "Private R&D forge — not a public GZMO SKU. Not on living GREEN.",
    "advice": (
        "okforge_observatory_RED — fix FAIL rows (forge/token/dual-writer/docs)"
        if verdict == "RED"
        else (
            "okforge_observatory_YELLOW — soft warnings or docs-only holds"
            if verdict == "YELLOW"
            else "okforge_observatory_GREEN — knowledge plane production bar"
        )
    ),
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n")
print(json.dumps({"verdict": verdict, "advice": payload["advice"], "pass": int(pass_n), "fail": int(fail_n), "warn": int(warn_n), "hold": int(hold_n)}, indent=2))
PY

[[ "$verdict" != "RED" ]]
