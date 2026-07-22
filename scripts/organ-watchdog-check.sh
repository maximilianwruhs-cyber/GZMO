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
import json, sys
from datetime import datetime, timezone
from pathlib import Path
data, out = Path(sys.argv[1]), Path(sys.argv[2])
wd = data / "scheduler-runs" / "latest-watchdog.json"
watch = {}
if wd.is_file():
    watch = json.loads(wd.read_text())
stale = bool(watch.get("stale"))
payload = {
    "schema": "gzmo.organ_watchdog.check/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "ok": True,
    "watchdog_stale": stale,
    "watchdog_detail": watch.get("detail"),
    "living": False,
    "advice": (
        "organ_watchdog_yellow — distill/dream stale (soft; does not flip GREEN math)"
        if stale
        else "organ_watchdog_ok — no stale soft-fail"
    ),
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n")
print(json.dumps(payload, indent=2))
PY
