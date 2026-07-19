#!/usr/bin/env bash
# Soft CT101 living probe for spine Keep lane.
# Records owner doctrine + optional SSH smoke; never hard-fails nightburst.
#
#   bash scripts/ct101-living-probe.sh
#   CT101_PROBE_SSH=0 bash scripts/ct101-living-probe.sh   # local checks only
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/ct101-living"
mkdir -p "$OUT"

SERVE_ACTIVE="unknown"
SCHED_ACTIVE="unknown"
if command -v systemctl >/dev/null 2>&1; then
  SERVE_ACTIVE="$(systemctl --user is-active gzmo-serve.service 2>/dev/null || true)"
  SCHED_ACTIVE="$(systemctl --user is-active gzmo-scheduler.service 2>/dev/null || true)"
  SERVE_ACTIVE="${SERVE_ACTIVE:-inactive}"
  SCHED_ACTIVE="${SCHED_ACTIVE:-inactive}"
  # systemctl prints "inactive" but exits non-zero — take first line only.
  SERVE_ACTIVE="$(printf '%s\n' "$SERVE_ACTIVE" | head -1)"
  SCHED_ACTIVE="$(printf '%s\n' "$SCHED_ACTIVE" | head -1)"
fi

SSH_MODE="${CT101_PROBE_SSH:-1}"
SMOKE_OK=0
SMOKE_NOTE="skipped"
SMOKE_LOG="$OUT/smoke.log"
if [[ "$SSH_MODE" == "1" ]]; then
  if bash "$ROOT/scripts/ct101-living-smoke.sh" >"$SMOKE_LOG" 2>&1; then
    SMOKE_OK=1
    SMOKE_NOTE="ct101-living-smoke PASS"
  else
    SMOKE_OK=0
    SMOKE_NOTE="ct101-living-smoke FAIL or unreachable (see smoke.log)"
  fi
else
  SMOKE_NOTE="CT101_PROBE_SSH=0 — local dual-writer check only"
fi

DUAL_WRITER_RISK=0
if [[ "$SERVE_ACTIVE" == "active" ]]; then
  DUAL_WRITER_RISK=1
fi

export OUT SERVE_ACTIVE SCHED_ACTIVE SMOKE_OK SMOKE_NOTE DUAL_WRITER_RISK
python3 - <<'PY'
import json, os
from datetime import datetime, timezone
from pathlib import Path

out = Path(os.environ["OUT"])
now = datetime.now(timezone.utc).isoformat()
smoke_ok = os.environ.get("SMOKE_OK") == "1"
dual = os.environ.get("DUAL_WRITER_RISK") == "1"
serve = os.environ.get("SERVE_ACTIVE")
sched = os.environ.get("SCHED_ACTIVE")
smoke_note = os.environ.get("SMOKE_NOTE")

local_ok = not dual
overall = local_ok and (smoke_ok or "SSH=0" in smoke_note or "unreachable" in smoke_note)
# Soft: unreachable SSH is HOLD for living proof but ok for lab nightburst
advice = (
    "living_ok — CT101 smoke PASS; workstation serve inactive"
    if smoke_ok and local_ok
    else (
        "dual_writer_risk — stop workstation gzmo-serve while CT101 lives"
        if dual
        else (
            "local_ok_ssh_hold — serve inactive; CT101 smoke not green"
            if local_ok
            else "hold"
        )
    )
)

payload = {
    "schema": "gzmo.ct101.living-probe/v1",
    "generated_at": now,
    "ok": True,  # soft for nightburst
    "living_proof": smoke_ok and local_ok,
    "advice": advice,
    "owner": {
        "living": "CT101 gzmo-daemon /opt/gzmo/",
        "lab": "workstation data-next/",
        "doc": "docs/CT101_BOUNDARY.md",
    },
    "workstation": {
        "gzmo_serve": serve,
        "gzmo_scheduler": sched,
        "dual_writer_risk": dual,
    },
    "ct101_smoke": {
        "ok": smoke_ok,
        "note": smoke_note,
        "log": str(out / "smoke.log"),
    },
    "note": "Keep-lane probe — protects sole overnight writer doctrine.",
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
(out / "latest.md").write_text(
    "\n".join(
        [
            "# CT101 living probe",
            "",
            f"Advice: **{advice}**",
            f"Workstation serve: `{serve}` · scheduler: `{sched}`",
            f"CT101 smoke: {smoke_note}",
            "",
            payload["note"],
            "",
        ]
    ),
    encoding="utf-8",
)
print(json.dumps({
    "ok": True,
    "living_proof": payload["living_proof"],
    "advice": advice,
    "serve": serve,
    "smoke_ok": smoke_ok,
}, indent=2))
PY
exit 0
