#!/usr/bin/env bash
# Unpark Wave 1.3: tinyFolder inbox → ingest spike readiness.
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

echo "=== tinyFolder check (Unpark W1.3) ==="
row PASS "inbox-dir" "$INBOX"
# Prefer documented ingest watcher / inbox path in config examples
if rg -n -i 'tinyfolder|inbox_ingest|inbox' "$ROOT/gzmo.toml.example" "$ROOT/docs" 2>/dev/null | head -1 >/dev/null; then
  row PASS "docs-hook" "inbox/ingest referenced in docs or example config"
else
  row HOLD "docs-hook" "no tinyFolder docs hit — spike only"
fi
# Soft: count pending drop files
n="$(find "$INBOX" -maxdepth 1 -type f 2>/dev/null | wc -l | tr -d ' ')"
row PASS "pending-files" "$n file(s) in inbox (0 ok)"
# Write a README so operators know the drop path
cat >"$INBOX/README.md" <<EOF
# tinyFolder inbox (Unpark Wave 1.3)

Drop markdown/text here for operator ingest experiments.
Does not auto-run overnight metabolism — pair with \`gzmo\` ingest / CT101 when ready.
EOF
row PASS "inbox-readme" "operator drop instructions"

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
advice="tinyfolder_ok" if fail_n==0 else "tinyfolder_fail"
payload={"schema":"gzmo.unpark.tinyfolder/v1","generated_at":datetime.now(timezone.utc).isoformat(),
  "verdict":verdict,"ok":fail_n==0,"advice":advice,"inbox":os.environ["INBOX"],
  "counts":{"pass":pass_n,"fail":fail_n,"hold":hold_n},"wave":"1.3","checks":checks}
(out/"latest.json").write_text(json.dumps(payload,indent=2)+"\n")
print(json.dumps({"verdict":verdict,"advice":advice,"pass":pass_n,"fail":fail_n,"hold":hold_n},indent=2))
raise SystemExit(0 if fail_n==0 else 1)
PY
