#!/usr/bin/env bash
# Append one keep-quality soak sample. Exit 0 only when gate GREEN.
# Unpark W1 waits until soak-log shows KEEP_QUALITY_SOAK_NIGHTS honest GREEN nights
# (min KEEP_QUALITY_SOAK_MIN_HOURS between counted samples — default 18h).
# Same-hour streaks → HOLD, not soak_ready_unpark_ok.
#
#   bash scripts/keep-quality-soak.sh
#   KEEP_QUALITY_SOAK_NIGHTS=3 bash scripts/keep-quality-soak.sh --summary
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/keep-quality"
LOG_JSONL="$OUT/soak-log.jsonl"
NIGHTS="${KEEP_QUALITY_SOAK_NIGHTS:-3}"
MIN_HOURS="${KEEP_QUALITY_SOAK_MIN_HOURS:-18}"
mkdir -p "$OUT"

if [[ "${1:-}" == "--summary" ]]; then
  python3 - <<PY
import json
from datetime import datetime, timezone
from pathlib import Path

p = Path("$LOG_JSONL")
n = int("$NIGHTS")
min_h = float("$MIN_HOURS")
min_sec = min_h * 3600.0

def parse_ts(row):
    raw = row.get("generated_at") or row.get("ts") or ""
    if not raw:
        return None
    try:
        return datetime.fromisoformat(raw.replace("Z", "+00:00"))
    except ValueError:
        return None

if not p.is_file():
    print(json.dumps({"ok": False, "advice": "no_soak_log", "need_nights": n, "min_hours": min_h}))
    raise SystemExit(1)

rows = [json.loads(l) for l in p.read_text().splitlines() if l.strip()]
greens = [r for r in rows if r.get("verdict") == "GREEN"]

# Raw trailing GREEN (any spacing) — for diagnostics
trail_raw = 0
for r in reversed(rows):
    if r.get("verdict") == "GREEN":
        trail_raw += 1
    else:
        break

# Honest trailing nights: walk newest→oldest; count GREEN only when ≥ min_h
# before the previously counted sample. Non-GREEN breaks the trail.
honest = 0
anchor = None  # datetime of last counted (newer) sample
spacing_rejects = 0
for r in reversed(rows):
    if r.get("verdict") != "GREEN":
        break
    ts = parse_ts(r)
    if ts is None:
        spacing_rejects += 1
        continue
    if anchor is None:
        honest += 1
        anchor = ts
        continue
    delta = (anchor - ts).total_seconds()
    if delta >= min_sec:
        honest += 1
        anchor = ts
    else:
        spacing_rejects += 1
        # same-hour / too-close: do not inflate night count; keep looking older

ready = honest >= n
if ready:
    advice = "soak_ready_unpark_ok"
elif trail_raw >= n and honest < n:
    advice = (
        f"soak_spacing_hold — trailing_GREEN={trail_raw} but honest_nights={honest} "
        f"(need {n} with ≥{min_h:g}h spacing; rejected_close={spacing_rejects})"
    )
else:
    advice = f"need_{n}_trailing_honest_GREEN_have_{honest}"

print(json.dumps({
    "ok": ready,
    "samples": len(rows),
    "green_total": len(greens),
    "trailing_green": trail_raw,
    "honest_nights": honest,
    "spacing_rejects": spacing_rejects,
    "min_hours": min_h,
    "need_nights": n,
    "advice": advice,
}, indent=2))
raise SystemExit(0 if ready else 1)
PY
  exit $?
fi

export LIVING_GATE_SKIP_TAKEAWAY="${LIVING_GATE_SKIP_TAKEAWAY:-1}"
set +e
bash "$ROOT/scripts/keep-quality-gate.sh"
rc=$?
set -e

python3 - <<PY
import json
from datetime import datetime, timezone
from pathlib import Path

out = Path("$OUT")
latest = out / "latest.json"
log = out / "soak-log.jsonl"
payload = {"generated_at": datetime.now(timezone.utc).isoformat(), "verdict": "RED", "ok": False}
if latest.is_file():
    payload.update(json.loads(latest.read_text()))
payload["soak_rc"] = int("$rc")
with log.open("a", encoding="utf-8") as f:
    f.write(json.dumps(payload, separators=(",", ":")) + "\n")
print(json.dumps({"appended": True, "verdict": payload.get("verdict"), "path": str(log)}, indent=2))
PY

# Print trailing summary
bash "$ROOT/scripts/keep-quality-soak.sh" --summary || true
exit "$rc"
