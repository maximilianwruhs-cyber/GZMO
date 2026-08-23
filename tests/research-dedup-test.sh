#!/usr/bin/env bash
# tests/research-dedup-test.sh — hermetic tests for scripts/lib-research-dedup.sh
# Cross-day URL dedup: dedup_findings / dedup_render_latest / dedup_seen_update.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# shellcheck source=scripts/lib-research-dedup.sh
source "$ROOT/scripts/lib-research-dedup.sh"

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

stamp="20260823T000000Z"

# Dates relative to today so the suite is hermetic on any calendar day.
read -r TODAY D5 D40 <<EOF
$(python3 - <<'PY'
from datetime import date, timedelta
t = date.today()
print(t.isoformat(), (t - timedelta(days=5)).isoformat(), (t - timedelta(days=40)).isoformat())
PY
)
EOF

FJ="$WORK/findings.json"
SJ="$WORK/seen.jsonl"

# Fixture: gzmo.research_intel/v1, 4 findings.
#   u1, u2 already in seen.jsonl (first_seen 5d and 40d ago) -> repeat.
#   u3, u4 not in seen -> new.
cat > "$FJ" <<JSON
{"schema":"gzmo.research_intel/v1","generated_at":"2026-08-23T00:00:00Z","ok":true,"queries":["q1"],"findings":[
{"source":"arxiv","title":"T1","url":"u1","published":"2026-08-01","benefit":true,"why":"w1","integration_point":"ip1"},
{"source":"github","title":"T2","url":"u2","published":"2026-07-01","benefit":false,"why":"","integration_point":""},
{"source":"arxiv","title":"T3","url":"u3","published":"2026-08-10","benefit":true,"why":"w3","integration_point":"ip3"},
{"source":"huggingface","title":"T4","url":"u4","published":"2026-08-12","benefit":false,"why":"","integration_point":""}
],"fetch_errors":null,"eval_error":null,"seen_total":2}
JSON

cat > "$SJ" <<JSONL
{"url":"u1","first_seen":"$D5"}
{"url":"u2","first_seen":"$D40"}
JSONL

fail() { echo "FAIL: $*" >&2; exit 1; }

# ── Group 1: dedup_findings ──────────────────────────────────────────────
SJ_BEFORE="$(cat "$SJ")"
OUT_STDOUT="$(dedup_findings "$FJ" "$SJ")"
[[ "$OUT_STDOUT" == '{"new":2,"repeat":2}' ]] || fail "dedup_findings stdout='$OUT_STDOUT'"
# seen file must be untouched by dedup_findings
[[ "$(cat "$SJ")" == "$SJ_BEFORE" ]] || fail "dedup_findings modified the seen file"
python3 - "$FJ" "$D5" "$D40" <<'PY' || fail "dedup_findings annotations/counts"
import json, sys
d = json.loads(open(sys.argv[1]).read())
fs = {f["url"]: f for f in d["findings"]}
ok = True
# repeats
for u in ("u1", "u2"):
    if fs[u].get("repeat") is not True:
        ok = False
    if "first_seen" not in fs[u]:
        ok = False
if fs["u1"].get("first_seen") != sys.argv[2]:
    ok = False
if fs["u2"].get("first_seen") != sys.argv[3]:
    ok = False
# new
for u in ("u3", "u4"):
    if fs[u].get("repeat") is not False:
        ok = False
    if "first_seen" in fs[u]:
        ok = False
if d.get("new_count") != 2 or d.get("repeat_count") != 2:
    ok = False
sys.exit(0 if ok else 1)
PY
echo "PASS dedup_findings"

# ── Group 2: dedup_render_latest (intel) ─────────────────────────────────
MD="$WORK/latest.md"
dedup_render_latest "$FJ" "$MD" intel "$stamp" 2
[[ -f "$MD" ]] || fail "render did not write md-out"
grep -F "findings: 4 (new: 2, repeat: 2)" "$MD" >/dev/null || fail "header findings line"
python3 - "$MD" <<'PY' || fail "render intel top/all sections"
import sys
md = open(sys.argv[1]).read()
top = md.split("## Top findings", 1)[1].split("## All findings", 1)[0]
allf = md.split("## All findings", 1)[1]
ok = True
# Top findings: only u3/u4 (new), never u1/u2 titles
if "### T3" not in top or "### T4" not in top:
    ok = False
if "### T1" in top or "### T2" in top:
    ok = False
# All findings: new -> " (new)", repeat -> " (repeat, first seen ...)"
if "- [arxiv] T3 — u3 (benefit=True) (new)" not in allf:
    ok = False
if "- [huggingface] T4 — u4 (benefit=False) (new)" not in allf:
    ok = False
if "(repeat, first seen " not in allf:
    ok = False
if "T1 — u1 (benefit=True) (repeat, first seen " not in allf:
    ok = False
if "T2 — u2 (benefit=False) (repeat, first seen " not in allf:
    ok = False
sys.exit(0 if ok else 1)
PY
echo "PASS dedup_render_latest intel"

# ── Group 3: dedup_seen_update ───────────────────────────────────────────
dedup_seen_update "$FJ" "$SJ" "$TODAY"
python3 - "$SJ" "$TODAY" <<'PY' || fail "seen update append/keep/prune"
import json, sys
today = sys.argv[2]
entries = [json.loads(l) for l in open(sys.argv[1]) if l.strip()]
m = {e["url"]: e["first_seen"] for e in entries}
ok = True
if "u1" not in m:           ok = False   # kept (5d, within 30d)
if "u2" in m:               ok = False   # pruned (40d > 30d)
if m.get("u3") != today:    ok = False   # appended new
if m.get("u4") != today:    ok = False   # appended new
sys.exit(0 if ok else 1)
PY
echo "PASS dedup_seen_update"

# idempotent: a second run with the same today appends nothing.
cp "$SJ" "$SJ.snap"
dedup_seen_update "$FJ" "$SJ" "$TODAY"
diff -q "$SJ.snap" "$SJ" >/dev/null || fail "seen update not idempotent"
echo "PASS dedup_seen_update idempotent"

echo "ALL TESTS PASSED"
