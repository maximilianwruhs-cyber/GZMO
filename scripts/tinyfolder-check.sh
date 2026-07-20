#!/usr/bin/env bash
# Brain Feed P0 / Unpark W1.3: tinyFolder inbox → living enqueue readiness.
#   bash scripts/tinyfolder-check.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/tinyfolder"
INBOX="${TINYFOLDER_INBOX:-$DATA/tinyfolder-inbox}"
mkdir -p "$OUT" "$INBOX"
pass=0; fail=0; hold=0
declare -a ROWS=()
row() { local s="$1" n="$2" d="$3"; ROWS+=("$s|$n|$d"); case "$s" in PASS) pass=$((pass+1));; FAIL) fail=$((fail+1));; HOLD) hold=$((hold+1));; esac; echo "[$s] $n — $d"; }

echo "=== tinyFolder check (Brain Feed P0) ==="
row PASS "inbox-dir" "$INBOX"
# Prefer documented ingest watcher / inbox path in config examples
if rg -n -i 'tinyfolder|inbox_ingest|inbox|BRAIN_FEED' "$ROOT/gzmo.toml.example" "$ROOT/docs" 2>/dev/null | head -1 >/dev/null; then
  row PASS "docs-hook" "inbox/ingest / Brain Feed referenced in docs"
else
  row HOLD "docs-hook" "no tinyFolder docs hit — spike only"
fi
# Soft: count pending drop files (README excluded from “operator drops”)
n="$(find "$INBOX" -maxdepth 1 -type f ! -name 'README.md' 2>/dev/null | wc -l | tr -d ' ')"
row PASS "pending-files" "$n drop file(s) in inbox (0 ok)"
# Write a README so operators know the drop path
cat >"$INBOX/README.md" <<EOF
# tinyFolder inbox (Brain Feed P0)

Drop markdown/text here for operator ingest experiments.
Does **not** start workstation overnight metabolism.
Brain Feed path: \`bash scripts/tinyfolder-drop.sh --demo --living\` → living-enqueue.json
aimed at CT101 / living host distill (\`gzmo:distill:pending\`).
EOF
row PASS "inbox-readme" "operator drop instructions"

# Brain Feed: living-enqueue artifact (dual-writer safe)
if [[ -f "$OUT/living-enqueue.json" ]]; then
  if python3 -c "
import json
d=json.load(open('$OUT/living-enqueue.json'))
ok=d.get('ok') is True and d.get('dual_writer') is False and d.get('living_distill_queue')=='gzmo:distill:pending'
raise SystemExit(0 if ok else 1)
"; then
    row PASS "living-enqueue" "$(python3 -c "import json;print(json.load(open('$OUT/living-enqueue.json')).get('advice',''))")"
  else
    row FAIL "living-enqueue" "living-enqueue.json not ok — dual_writer or missing queue"
  fi
else
  row HOLD "living-enqueue" "no living-enqueue.json — bash scripts/tinyfolder-drop.sh --demo --living"
fi

# Demo evidence: sample drop + dry-run log (lab only; never product vault / overnight)
if [[ -f "$OUT/demo.json" ]]; then
  if python3 -c "
import json
from pathlib import Path
d=json.load(open('$OUT/demo.json'))
sample=Path(d.get('sample') or '')
log=Path(d.get('dry_run_log') or '')
ok=bool(d.get('ok')) and sample.is_file() and log.is_file() and log.stat().st_size>0
raise SystemExit(0 if ok else 1)
"; then
    row PASS "drop-demo" "$(python3 -c "import json;print(json.load(open('$OUT/demo.json')).get('sample',''))")"
  else
    row FAIL "drop-demo" "demo.json incomplete — rerun tinyfolder-ingest-demo.sh"
  fi
else
  row HOLD "drop-demo" "no demo.json yet — bash scripts/tinyfolder-ingest-demo.sh"
fi

ROWS_TSV="$(printf '%s\n' "${ROWS[@]}")"
export OUT pass fail hold ROWS_TSV INBOX
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
advice = "tinyfolder_ok" if fail_n==0 and checks.get("living-enqueue",{}).get("status") in ("PASS","HOLD") else (
    "tinyfolder_hold — run --living drop" if fail_n==0 else "tinyfolder_fail")
if fail_n==0 and checks.get("living-enqueue",{}).get("status")=="PASS":
    advice = "tinyfolder_living_ok"
payload={"schema":"gzmo.brain_feed.tinyfolder/v1","generated_at":datetime.now(timezone.utc).isoformat(),
  "verdict":verdict,"ok":fail_n==0,"advice":advice,"inbox":os.environ["INBOX"],
  "counts":{"pass":pass_n,"fail":fail_n,"hold":hold_n},"wave":"brain_feed_p0","checks":checks}
(out/"latest.json").write_text(json.dumps(payload,indent=2)+"\n")
print(json.dumps({"verdict":verdict,"advice":advice,"pass":pass_n,"fail":fail_n,"hold":hold_n},indent=2))
raise SystemExit(0 if fail_n==0 else 1)
PY
