#!/usr/bin/env bash
# Unpark Wave 4.3: Wiki + Observatory theater readiness (search + sanitized scoreboard).
#   bash scripts/wiki-observatory-check.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/wiki-observatory"
mkdir -p "$OUT"
pass=0; fail=0; hold=0
declare -a ROWS=()
row() { local s="$1" n="$2" d="$3"; ROWS+=("$s|$n|$d"); case "$s" in PASS) pass=$((pass+1));; FAIL) fail=$((fail+1));; HOLD) hold=$((hold+1));; esac; echo "[$s] $n — $d"; }

echo "=== Wiki / Observatory check (Unpark W4.3) ==="
[[ -f "$ROOT/docs/WIKI_OBSERVATORY_MIND.md" ]] && row PASS "mind-doc" "WIKI_OBSERVATORY_MIND.md" || row FAIL "mind-doc" "missing"
[[ -f "$ROOT/docs/WIKI_OBSERVATORY_DEMO.md" ]] && row PASS "demo-door" "WIKI_OBSERVATORY_DEMO.md" || row HOLD "demo-door" "missing theater front door"
[[ -f "$ROOT/docs/WIKI_LAYER.md" ]] && row PASS "wiki-layer" "WIKI_LAYER.md" || row HOLD "wiki-layer" "missing"

[[ -x "$ROOT/scripts/wiki-observatory-demo.sh" ]] && row PASS "demo-script" "wiki-observatory-demo.sh" || row FAIL "demo-script" "missing"
[[ -x "$ROOT/scripts/wiki-mind-check.sh" ]] && row PASS "mind-check" "wiki-mind-check.sh" || row FAIL "mind-check" "missing"
[[ -x "$ROOT/scripts/nightburst-scoreboard.sh" ]] && row PASS "scoreboard-script" "nightburst-scoreboard.sh" || row FAIL "scoreboard-script" "missing"

[[ -d "$ROOT/wiki" ]] && row PASS "wiki-dir" "wiki/" || row FAIL "wiki-dir" "missing"

if [[ -f "$DATA/wiki-mind/latest.json" ]]; then
  if python3 -c "
import json
d=json.load(open('$DATA/wiki-mind/latest.json'))
raise SystemExit(0 if d.get('ok') is True else 1)
"; then
    row PASS "wiki-mind" "seeded search GREEN"
  else
    row FAIL "wiki-mind" "wiki-mind latest not ok — rerun wiki-mind-check"
  fi
else
  row HOLD "wiki-mind" "no wiki-mind/latest.json — run wiki-observatory-demo"
fi

if [[ -f "$DATA/arena/scoreboard.json" && -f "$DATA/arena/scoreboard.html" ]]; then
  if python3 -c "
import json
d=json.load(open('$DATA/arena/scoreboard.json'))
ok=(d.get('schema')=='gzmo.nightburst.scoreboard/v1' and isinstance(d.get('wiki'), dict))
# Sanitized: no obvious secret keys
blob=json.dumps(d).lower()
bad=any(k in blob for k in ('api_key','authorization','bearer ','password='))
raise SystemExit(0 if ok and not bad else 1)
"; then
    row PASS "scoreboard" "sanitized scoreboard.json + html"
  else
    row FAIL "scoreboard" "scoreboard missing schema/wiki or looks unsanitized"
  fi
else
  row HOLD "scoreboard" "run nightburst-scoreboard / wiki-observatory-demo"
fi

if rg -n 'wiki-mind|wiki-observatory' "$ROOT/scripts/living-readiness-gate.sh" >/dev/null 2>&1; then
  row FAIL "not-living-required" "wiki observatory wired into living-readiness — remove"
else
  row PASS "not-living-required" "living gate independent of wiki/observatory theater"
fi

# Demo must not claim a push happened
if [[ -f "$OUT/demo.json" ]]; then
  if python3 -c "
import json
d=json.load(open('$OUT/demo.json'))
ok=(
  d.get('schema')=='gzmo.unpark.wiki_observatory.demo/v1'
  and d.get('ok') is True
  and d.get('daemon_jobs_touched') is False
  and d.get('wiki_push_applied') is False
)
raise SystemExit(0 if ok else 1)
"; then
    row PASS "demo-inventory" "demo.json — search+scoreboard; no push"
  else
    row FAIL "demo-inventory" "demo.json incomplete — rerun wiki-observatory-demo.sh"
  fi
else
  row HOLD "demo-inventory" "no demo.json yet — bash scripts/wiki-observatory-demo.sh"
fi

if [[ -f "$OUT/felt-latest.md" ]]; then
  row PASS "felt" "felt-latest.md"
else
  row HOLD "felt" "no felt-latest yet"
fi

ROWS_TSV="$(printf '%s\n' "${ROWS[@]}")"
export OUT pass fail hold ROWS_TSV
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
demo_ok = checks.get("demo-inventory",{}).get("status")=="PASS"
if fail_n==0 and demo_ok:
    advice="wiki_observatory_ok — mind search + sanitized scoreboard theater"
elif fail_n==0:
    advice="wiki_observatory_hold — run wiki-observatory-demo.sh"
else:
    advice="wiki_observatory_fail"
payload={"schema":"gzmo.unpark.wiki_observatory/v1","generated_at":datetime.now(timezone.utc).isoformat(),
  "verdict":verdict,"ok":fail_n==0,"advice":advice,"demo":demo_ok,
  "counts":{"pass":pass_n,"fail":fail_n,"hold":hold_n},"wave":"4.3","checks":checks,
  "note":"Not on living GREEN overnight gate. OKForge /observatory stays agent-discovery."}
(out/"latest.json").write_text(json.dumps(payload,indent=2)+"\n")
print(json.dumps({"verdict":verdict,"advice":advice,"pass":pass_n,"fail":fail_n,"hold":hold_n},indent=2))
raise SystemExit(0 if fail_n==0 else 1)
PY
