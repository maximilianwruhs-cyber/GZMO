#!/usr/bin/env bash
# Promote-loop living-apply overnight soak check (Done when #4).
# Does not apply loops. Writes a receipt under data-next/beat-gate/promotions/.
#
#   bash scripts/promote-loop-soak-check.sh
#   PROMOTE_SOAK_MIN_HOURS=12 bash scripts/promote-loop-soak-check.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/beat-gate/promotions"
HOST="${CT101_SSH_HOST:-ct101}"
MIN_H="${PROMOTE_SOAK_MIN_HOURS:-12}"
mkdir -p "$OUT"

pass=0; fail=0; hold=0
declare -a ROWS=()
row() { local s="$1" n="$2" d="$3"; ROWS+=("$s|$n|$d"); case "$s" in PASS) pass=$((pass+1));; FAIL) fail=$((fail+1));; HOLD) hold=$((hold+1));; esac; echo "[$s] $n — $d"; }

echo "=== Promote-loop overnight soak check ==="
row PASS "no-apply" "this script never PROMOTE_APPLY / never writes living toml"

# Dual-writer
SERVE="$(systemctl --user is-active gzmo-serve.service 2>/dev/null || true)"
SERVE="$(printf '%s\n' "$SERVE" | head -1)"
if [[ "$SERVE" == "active" ]]; then
  row FAIL "dual-writer" "gzmo-serve active — refuse soak claim"
else
  row PASS "dual-writer" "serve=${SERVE:-inactive}"
fi

# Pull pin ages from CT101
PIN_JSON="$OUT/ct101-living-applied-snapshot.json"
if ssh -o ConnectTimeout=12 -o BatchMode=yes "$HOST" \
  "python3 - <<'PY'
import json
from pathlib import Path
base = Path('/opt/gzmo/data/beat-gate/promotions')
out = {}
for name in ('living-applied-knowledge.json','living-applied-cognition.json','living-applied.json'):
    p = base / name
    if p.is_file():
        out[name] = json.loads(p.read_text())
print(json.dumps(out))
PY" >"$PIN_JSON" 2>/tmp/promote-soak-ssh.err; then
  row PASS "pins-fetch" "CT101 living-applied snapshot"
else
  row FAIL "pins-fetch" "ssh ct101 failed — see /tmp/promote-soak-ssh.err"
fi

export OUT PIN_JSON MIN_H pass fail hold
ROWS_TSV="$(printf '%s\n' "${ROWS[@]}")"
export ROWS_TSV ROOT DATA HOST

# Brain Feed + keep-quality (soft append not done here)
set +e
bash "$ROOT/scripts/brain-feed-check.sh" >/tmp/promote-soak-bf.log 2>&1
BF_RC=$?
bash "$ROOT/scripts/keep-quality-gate.sh" >/tmp/promote-soak-kq.log 2>&1
KQ_RC=$?
set -e

python3 - <<'PY'
import json, os
from datetime import datetime, timezone
from pathlib import Path

out = Path(os.environ["OUT"])
pin_path = Path(os.environ["PIN_JSON"])
min_h = float(os.environ["MIN_H"])
rows = []
for line in os.environ.get("ROWS_TSV", "").splitlines():
    if not line.strip():
        continue
    st, n, d = line.split("|", 2)
    rows.append({"status": st, "name": n, "detail": d})

def row(status, name, detail):
    rows.append({"status": status, "name": name, "detail": detail})

bf_rc = int(os.environ.get("BF_RC", "1") or "1")
kq_rc = int(os.environ.get("KQ_RC", "1") or "1")
# re-read from env set below — pass via files
bf_rc = 0 if Path("/tmp/promote-soak-bf.log").exists() else 1

# Parse BF/KQ from exit files written by shell — use env vars injected after
# Actually shell didn't export BF_RC into python env before heredoc started.
# Re-check from latest.json artifacts instead.
data = Path(os.environ["DATA"])
bf = {}
kq = {}
try:
    bf = json.loads((data / "brain-feed" / "latest.json").read_text())
except Exception:
    bf = {}
try:
    kq = json.loads((data / "keep-quality" / "latest.json").read_text())
except Exception:
    kq = {}

if bf.get("verdict") == "GREEN" or bf.get("ok") is True:
    row("PASS", "brain-feed", "Brain Feed GREEN")
else:
    row("FAIL", "brain-feed", f"Brain Feed not GREEN ({bf.get('verdict') or bf.get('advice')})")

if kq.get("verdict") == "GREEN" or kq.get("ok") is True:
    row("PASS", "keep-quality", "keep-quality GREEN")
else:
    row("FAIL", "keep-quality", f"keep-quality not GREEN ({kq.get('verdict') or kq.get('advice')})")

pins = {}
if pin_path.is_file():
    try:
        pins = json.loads(pin_path.read_text())
    except Exception as e:
        row("FAIL", "pins-parse", str(e))

now = datetime.now(timezone.utc)
ages = {}
oldest = None
for name, payload in pins.items():
    raw = payload.get("applied_at") or payload.get("generated_at") or payload.get("pinned_at")
    loop = payload.get("loop") or payload.get("loops") or name
    if not raw:
        ages[name] = {"loop": loop, "hours": None, "applied_at": None}
        continue
    try:
        ts = datetime.fromisoformat(str(raw).replace("Z", "+00:00"))
    except ValueError:
        ages[name] = {"loop": loop, "hours": None, "applied_at": raw}
        continue
    hours = (now - ts).total_seconds() / 3600.0
    ages[name] = {"loop": loop, "hours": round(hours, 3), "applied_at": raw}
    if oldest is None or ts < oldest:
        oldest = ts

overnight_ok = False
min_age_h = None
if ages:
    known = [a["hours"] for a in ages.values() if a.get("hours") is not None]
    if known:
        min_age_h = min(known)
        overnight_ok = min_age_h >= min_h

if overnight_ok:
    row("PASS", "overnight-age", f"min pin age {min_age_h:.2f}h ≥ {min_h:g}h")
elif min_age_h is None:
    row("HOLD", "overnight-age", "no pin timestamps — apply first")
else:
    row(
        "HOLD",
        "overnight-age",
        f"min pin age {min_age_h:.2f}h < {min_h:g}h — wait for overnight metabolism",
    )

fail_n = sum(1 for r in rows if r["status"] == "FAIL")
hold_n = sum(1 for r in rows if r["status"] == "HOLD")
pass_n = sum(1 for r in rows if r["status"] == "PASS")
soaked = fail_n == 0 and overnight_ok and (bf.get("verdict") == "GREEN" or bf.get("ok") is True) and (
    kq.get("verdict") == "GREEN" or kq.get("ok") is True
)

if soaked:
    advice = "promote_loop_soak_ok — Done when #4 met; mark promote-loop-living-apply soaked"
    verdict = "GREEN"
elif fail_n:
    advice = "promote_loop_soak_fail — fix FAIL rows"
    verdict = "RED"
else:
    advice = "promote_loop_soak_hold — wait overnight or re-check BF/keep-quality"
    verdict = "HOLD"

payload = {
    "schema": "gzmo.promote_loop.soak/v1",
    "generated_at": now.isoformat(),
    "verdict": verdict,
    "ok": soaked,
    "advice": advice,
    "auto_apply": False,
    "min_hours_required": min_h,
    "min_pin_age_hours": min_age_h,
    "pins": ages,
    "brain_feed": {"verdict": bf.get("verdict"), "ok": bf.get("ok")},
    "keep_quality": {"verdict": kq.get("verdict"), "ok": kq.get("ok")},
    "counts": {"pass": pass_n, "fail": fail_n, "hold": hold_n},
    "checks": {r["name"]: {"status": r["status"], "detail": r["detail"]} for r in rows},
    "operator": [
        "If HOLD overnight-age: leave CT101 daemon running through the night",
        "Re-run: bash scripts/promote-loop-soak-check.sh",
        "On GREEN: update research/opportunities/promote-loop-living-apply.md status=soaked",
    ],
}
(out / "soak-latest.json").write_text(json.dumps(payload, indent=2) + "\n")
md = [
    "# Promote-loop soak check",
    "",
    f"Verdict: **{verdict}**",
    "",
    f"- Advice: {advice}",
    f"- Min pin age (h): `{min_age_h}` (need ≥{min_h:g})",
    f"- Brain Feed: `{bf.get('verdict')}`",
    f"- Keep-quality: `{kq.get('verdict')}`",
    "",
]
for r in rows:
    md.append(f"- [{r['status']}] {r['name']} — {r['detail']}")
(out / "soak-latest.md").write_text("\n".join(md) + "\n", encoding="utf-8")
print(json.dumps({"verdict": verdict, "ok": soaked, "advice": advice, "min_pin_age_hours": min_age_h}, indent=2))
raise SystemExit(0 if fail_n == 0 else 1)
PY
