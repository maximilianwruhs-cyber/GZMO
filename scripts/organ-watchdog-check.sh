#!/usr/bin/env bash
# O13 — organ-trace + missed-run watchdog soft surface (local).
#   bash scripts/organ-watchdog-check.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/organ-watchdog"
mkdir -p "$OUT"

# Soft: prefer local scheduler-runs; --living only when CT101_SSH_HOST set and ORGAN_LIVING=1
if [[ "${ORGAN_LIVING:-0}" == "1" ]]; then
  bash "$ROOT/scripts/organ-trace.sh" --living 2>&1 | tee "$OUT/trace.log" | tail -30 || true
else
  bash "$ROOT/scripts/organ-trace.sh" 2>&1 | tee "$OUT/trace.log" | tail -30 || true
fi

python3 - "$DATA" "$OUT" <<'PY'
import json, os, sys
from datetime import datetime, timezone
from pathlib import Path
data, out = Path(sys.argv[1]), Path(sys.argv[2])
living = os.environ.get("ORGAN_LIVING", "0") == "1"
# Living mode: trust ONLY the fresh CT101 mirror written by organ-trace --living;
# the lab data-next/scheduler-runs copy can be days stale (last-run residue), so
# a missing mirror means "no fresh evidence", not "use the stale lab copy".
mirror = data / "organ-trace" / "living-scheduler-runs" / "latest-watchdog.json"
lab = data / "scheduler-runs" / "latest-watchdog.json"
wd = mirror if living else lab
watch = {}
if wd.is_file():
    try:
        watch = json.loads(wd.read_text())
    except Exception:
        watch = {}  # truncated scp must not abort the soft watchdog
stale = bool(watch.get("stale"))
payload = {
    "schema": "gzmo.organ_watchdog.check/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "ok": True,
    "watchdog_stale": stale,
    "watchdog_detail": watch.get("detail"),
    "watchdog_source": str(wd) if wd.is_file() else "missing",
    "living": living,
    "advice": (
        "organ_watchdog_yellow — distill/dream stale (soft; does not flip GREEN math)"
        if stale
        else "organ_watchdog_ok — no stale soft-fail"
    ),
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n")
print(json.dumps(payload, indent=2))
PY
