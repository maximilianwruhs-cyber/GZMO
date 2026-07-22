#!/usr/bin/env bash
# O9 — record a human pin decision (accept / reject / defer). Never applies toml.
#   bash scripts/brain-intel-pin-log.sh --decision accept --reason "verify_pass_rate improved"
#   bash scripts/brain-intel-pin-log.sh --decision reject --reason "Arena estimate-only joules"
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/brain-intel"
DECISION=""
REASON=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --decision) DECISION="${2:-}"; shift 2 ;;
    --reason) REASON="${2:-}"; shift 2 ;;
    *) shift ;;
  esac
done
[[ -n "$DECISION" && -n "$REASON" ]] || {
  echo "usage: $0 --decision accept|reject|defer --reason '…'" >&2
  exit 2
}
case "$DECISION" in accept|reject|defer) ;; *)
  echo "decision must be accept|reject|defer" >&2; exit 2 ;;
esac
mkdir -p "$OUT/pins"
export OUT DECISION REASON
python3 - <<'PY'
import json, os
from datetime import datetime, timezone
from pathlib import Path
out = Path(os.environ["OUT"])
stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
row = {
    "schema": "gzmo.brain_intel.pin_decision/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "decision": os.environ["DECISION"],
    "reason": os.environ["REASON"],
    "auto_apply": False,
    "note": "log only — living toml unchanged",
}
path = out / "pins" / f"pin-{os.environ['DECISION']}-{stamp}.json"
path.write_text(json.dumps(row, indent=2) + "\n")
# rollup
pins = sorted((out / "pins").glob("pin-*.json"))
accepted = rejected = deferred = 0
for p in pins:
    d = json.loads(p.read_text()).get("decision")
    if d == "accept": accepted += 1
    elif d == "reject": rejected += 1
    elif d == "defer": deferred += 1
rollup = {
    "schema": "gzmo.brain_intel.pin_log/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "ok": accepted >= 1 and rejected >= 1,
    "accepted": accepted,
    "rejected": rejected,
    "deferred": deferred,
    "latest": str(path),
    "advice": (
        "pin_log_ok — ≥1 accept and ≥1 reject recorded"
        if accepted >= 1 and rejected >= 1
        else "pin_log_partial — need ≥1 accept and ≥1 reject for O9"
    ),
}
(out / "pin-log-latest.json").write_text(json.dumps(rollup, indent=2) + "\n")
print(json.dumps(rollup, indent=2))
raise SystemExit(0 if rollup["ok"] else 0)  # recording always succeeds
PY
