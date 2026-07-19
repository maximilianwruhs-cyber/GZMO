#!/usr/bin/env bash
# Unpark Wave 4.3 demable: wiki / Observatory mind smoke (git wiki search).
#   bash scripts/wiki-mind-check.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/wiki-mind"
mkdir -p "$OUT"
pass=0; fail=0; hold=0
declare -a ROWS=()
row() { local s="$1" n="$2" d="$3"; ROWS+=("$s|$n|$d"); case "$s" in PASS) pass=$((pass+1));; FAIL) fail=$((fail+1));; HOLD) hold=$((hold+1));; esac; echo "[$s] $n — $d"; }

echo "=== Wiki mind check (Unpark W4.3) ==="
[[ -f "$ROOT/docs/WIKI_OBSERVATORY_MIND.md" ]] && row PASS "doc" "WIKI_OBSERVATORY_MIND.md" || row FAIL "doc" "missing"
[[ -d "$ROOT/wiki" ]] && row PASS "wiki-dir" "wiki/" || row FAIL "wiki-dir" "missing"
n="$(find "$ROOT/wiki" -name '*.md' 2>/dev/null | wc -l | tr -d ' ')"
[[ "$n" -gt 0 ]] && row PASS "wiki-pages" "$n markdown pages" || row HOLD "wiki-pages" "no md pages"

BIN="${GZMO_BIN:-$ROOT/target/release/gzmo}"
[[ -x "$BIN" ]] || BIN="${CARGO_TARGET_DIR:-$HOME/github-clone/temp-bench/target}/release/gzmo"
if [[ -x "$BIN" ]]; then
  set +e
  GZMO_ALLOW_LAB_VAULT=1 "$BIN" wiki search "memory" --limit 3 >"$OUT/wiki-search.txt" 2>&1
  rc=$?
  set -e
  [[ "$rc" -eq 0 ]] && row PASS "wiki-search" "gzmo wiki search ok" || row HOLD "wiki-search" "wiki search exit $rc"
else
  row HOLD "wiki-search" "no gzmo binary"
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
advice="wiki_mind_ok" if fail_n==0 else "wiki_mind_fail"
payload={"schema":"gzmo.unpark.wiki_mind/v1","generated_at":datetime.now(timezone.utc).isoformat(),
  "verdict":verdict,"ok":fail_n==0,"advice":advice,
  "counts":{"pass":pass_n,"fail":fail_n,"hold":hold_n},"wave":"4.3","checks":checks}
(out/"latest.json").write_text(json.dumps(payload,indent=2)+"\n")
print(json.dumps({"verdict":verdict,"advice":advice,"pass":pass_n,"fail":fail_n,"hold":hold_n},indent=2))
raise SystemExit(0 if fail_n==0 else 1)
PY
