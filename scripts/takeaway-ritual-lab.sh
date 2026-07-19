#!/usr/bin/env bash
# Keep-lane lab: session close --takeaway → session [TAKEAWAY] + distill enqueue.
# Does not run distill --now (no overnight metabolism on workstation).
#
#   bash scripts/takeaway-ritual-lab.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/takeaway-ritual"
SESSIONS="${GZMO_SESSIONS_DIR:-$DATA/sessions}"
QUEUE="${GZMO_DISTILL_FALLBACK:-$DATA/distill-queue}"
BIN="${GZMO_BIN:-${CARGO_TARGET_DIR:-$HOME/github-clone/temp-bench/target}/release/gzmo}"
export GZMO_INSTANCE="${GZMO_INSTANCE:-next}"
export GZMO_CONFIG="${GZMO_CONFIG:-$ROOT/config/gzmo.toml}"
export GZMO_ALLOW_LAB_VAULT="${GZMO_ALLOW_LAB_VAULT:-1}"

mkdir -p "$OUT" "$SESSIONS" "$QUEUE"

MARKER="SpineTakeaway-$(date -u +%Y%m%dT%H%M%SZ)-$$"
SID="takeaway-lab-$$"
NOW="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

cat >"$SESSIONS/${SID}.json" <<EOF
{
  "id": "${SID}",
  "name": "takeaway_ritual_lab",
  "created_at": "${NOW}",
  "last_active_at": "${NOW}",
  "messages": [
    {
      "role": "user",
      "content": "Lab session for takeaway ritual proof.",
      "is_meta": false
    },
    {
      "role": "assistant",
      "content": "Ready to record a durable takeaway.",
      "is_meta": false
    }
  ]
}
EOF

CLOSE_LOG="$OUT/close.log"
CLOSE_OK=0
if [[ -x "$BIN" ]]; then
  if "$BIN" session close "$SID" --takeaway "$MARKER: session close feeds distill queue" \
    >"$CLOSE_LOG" 2>&1; then
    CLOSE_OK=1
  else
    CLOSE_OK=0
  fi
else
  echo "no gzmo binary at $BIN" >"$CLOSE_LOG"
fi

export OUT SID MARKER SESSIONS QUEUE CLOSE_OK CLOSE_LOG BIN
python3 - <<'PY'
import json, os
from datetime import datetime, timezone
from pathlib import Path

out = Path(os.environ["OUT"])
sid = os.environ["SID"]
marker = os.environ["MARKER"]
sessions = Path(os.environ["SESSIONS"])
queue = Path(os.environ["QUEUE"])
close_ok = os.environ.get("CLOSE_OK") == "1"
now = datetime.now(timezone.utc).isoformat()

session_path = sessions / f"{sid}.json"
session_has = False
takeaway_lines = 0
if session_path.is_file():
    text = session_path.read_text(encoding="utf-8", errors="replace")
    session_has = f"[TAKEAWAY] {marker}" in text or f"[TAKEAWAY]{marker}" in text or marker in text and "[TAKEAWAY]" in text
    takeaway_lines = text.count("[TAKEAWAY]")

queued = False
queue_hits = []
if queue.is_dir():
    for p in sorted(queue.glob("*.json"), key=lambda x: x.stat().st_mtime, reverse=True)[:40]:
        try:
            raw = p.read_text(encoding="utf-8", errors="replace")
        except Exception:
            continue
        if sid in raw or marker in raw:
            queued = True
            queue_hits.append(str(p))
            break

# Also accept close log saying enqueued even if Redis ate the job (no file fallback).
log = Path(os.environ["CLOSE_LOG"]).read_text(encoding="utf-8", errors="replace")
enqueued_msg = "enqueued" in log.lower() or "distill job" in log.lower()

ritual_ok = close_ok and session_has and (queued or enqueued_msg)
advice = (
    "ritual_ok — takeaway appended and distill path engaged (enqueue; no --now)"
    if ritual_ok
    else (
        "partial — close ran but queue/session evidence incomplete"
        if close_ok
        else "hold — session close failed (see close.log)"
    )
)

payload = {
    "schema": "gzmo.takeaway.ritual/v1",
    "generated_at": now,
    "ok": True,  # soft for nightburst
    "ritual_ok": ritual_ok,
    "advice": advice,
    "session_id": sid,
    "marker": marker,
    "close_ok": close_ok,
    "session_has_takeaway": session_has,
    "takeaway_lines": takeaway_lines,
    "queue_hit": queued,
    "queue_paths": queue_hits,
    "enqueued_msg": enqueued_msg,
    "bin": os.environ.get("BIN"),
    "production": {
        "ran_distill_now": False,
        "note": "Enqueue only — metabolize via CT101 overnight or explicit lab distill",
    },
    "note": "Keep-lane human loop proof — close the session so metabolism has fuel.",
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
(out / "latest.md").write_text(
    "\n".join(
        [
            "# Takeaway ritual lab",
            "",
            f"Advice: **{advice}**",
            f"Session: `{sid}`",
            f"Marker: `{marker}`",
            f"Session [TAKEAWAY]: {session_has} ({takeaway_lines} lines)",
            f"Queue hit: {queued} · enqueued msg: {enqueued_msg}",
            "",
            payload["note"],
            "",
        ]
    ),
    encoding="utf-8",
)
print(json.dumps({"ok": True, "ritual_ok": ritual_ok, "advice": advice, "session_id": sid}, indent=2))
PY
exit 0
