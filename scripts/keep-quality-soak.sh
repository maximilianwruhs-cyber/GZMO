#!/usr/bin/env bash
# Append one keep-quality soak sample. Exit 0 only when gate GREEN.
# Unpark W1 waits until soak-log shows KEEP_QUALITY_SOAK_NIGHTS consecutive GREEN nights
# (operator runs this once per night, or after metabolism cycles).
#
#   bash scripts/keep-quality-soak.sh
#   KEEP_QUALITY_SOAK_NIGHTS=3 bash scripts/keep-quality-soak.sh --summary
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/keep-quality"
LOG_JSONL="$OUT/soak-log.jsonl"
NIGHTS="${KEEP_QUALITY_SOAK_NIGHTS:-3}"
mkdir -p "$OUT"

if [[ "${1:-}" == "--summary" ]]; then
  python3 - <<PY
import json
from pathlib import Path
p = Path("$LOG_JSONL")
n = int("$NIGHTS")
if not p.is_file():
    print(json.dumps({"ok": False, "advice": "no_soak_log", "need_nights": n}))
    raise SystemExit(1)
rows = [json.loads(l) for l in p.read_text().splitlines() if l.strip()]
greens = [r for r in rows if r.get("verdict") == "GREEN"]
# consecutive trailing GREEN
trail = 0
for r in reversed(rows):
    if r.get("verdict") == "GREEN":
        trail += 1
    else:
        break
ready = trail >= n
print(json.dumps({
    "ok": ready,
    "samples": len(rows),
    "green_total": len(greens),
    "trailing_green": trail,
    "need_nights": n,
    "advice": "soak_ready_unpark_ok" if ready else f"need_{n}_trailing_GREEN_have_{trail}",
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
