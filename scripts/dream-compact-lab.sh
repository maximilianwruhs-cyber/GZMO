#!/usr/bin/env bash
# Keep-lane: dream compact dry-run (GC plumbing; never on GREEN overnight gate).
#
#   bash scripts/dream-compact-lab.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/dream-compact"
BIN="${GZMO_BIN:-${CARGO_TARGET_DIR:-$HOME/github-clone/temp-bench/target}/release/gzmo}"
export GZMO_INSTANCE="${GZMO_INSTANCE:-next}"
export GZMO_CONFIG="${GZMO_CONFIG:-$ROOT/config/gzmo.toml}"
export GZMO_ALLOW_LAB_VAULT="${GZMO_ALLOW_LAB_VAULT:-1}"
mkdir -p "$OUT"

LOG="$OUT/dry-run.log"
OK=0
NOTE="skipped"
if [[ -x "$BIN" ]]; then
  if "$BIN" dream compact --dry-run >"$LOG" 2>&1; then
    OK=1
    NOTE="dream compact --dry-run PASS"
  else
    OK=0
    NOTE="dream compact --dry-run FAIL (see dry-run.log)"
  fi
else
  NOTE="no gzmo binary"
  echo "$NOTE" >"$LOG"
fi

export OUT OK NOTE LOG
python3 - <<'PY'
import json, os, re
from datetime import datetime, timezone
from pathlib import Path

out = Path(os.environ["OUT"])
ok = os.environ.get("OK") == "1"
note = os.environ["NOTE"]
log = Path(os.environ["LOG"]).read_text(encoding="utf-8", errors="replace")
now = datetime.now(timezone.utc).isoformat()

# Best-effort parse of common report fields from CLI output.
chars = None
m = re.search(r"(\d+)\s*→\s*(\d+)", log)
before = after = None
if m:
    before, after = int(m.group(1)), int(m.group(2))
advice = (
    "compact_dry_ok — GC path alive; schedule stays soft Sunday / not on GREEN"
    if ok
    else "hold — dream compact dry-run failed"
)

payload = {
    "schema": "gzmo.dream-compact.lab/v1",
    "generated_at": now,
    "ok": True,
    "dry_run_ok": ok,
    "advice": advice,
    "note_cli": note,
    "chars_before": before,
    "chars_after": after,
    "log_tail": "\n".join(log.strip().splitlines()[-12:]),
    "production": {
        "on_green_overnight": False,
        "mutate": False,
        "note": "Dry-run only — real compact is serve soft-fail Sunday path.",
    },
    "note": "Keep plumbing — protect DREAMS.md size without overnight drama.",
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
(out / "latest.md").write_text(
    "\n".join(
        [
            "# Dream compact lab",
            "",
            f"Advice: **{advice}**",
            f"CLI: {note}",
            "",
            payload["note"],
            "",
            "```",
            payload["log_tail"],
            "```",
            "",
        ]
    ),
    encoding="utf-8",
)
print(json.dumps({"ok": True, "dry_run_ok": ok, "advice": advice}, indent=2))
PY
exit 0
