#!/usr/bin/env bash
# Unpark Wave 1.1 demable: link herdr + takeaway → session close → distill enqueue (lab).
# Never passes --now (no overnight metabolism on workstation / no CT101 force).
#
#   bash scripts/herdr-metabolism-demo.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/herdr-metabolism"
SESSIONS="${GZMO_SESSIONS_DIR:-$DATA/sessions}"
QUEUE="${GZMO_DISTILL_FALLBACK:-$DATA/distill-queue}"
PLUGIN="$ROOT/integrations/herdr-gzmo-metabolism"
gzmo_has_session_close() {
  local cand="$1" help
  [[ -x "$cand" ]] || return 1
  # Help exits 2 (clap); capture must ignore that under pipefail. REPL stubs lack takeaway.
  help="$("$cand" session close --help 2>&1 || true)"
  grep -qi 'takeaway' <<<"$help"
}

BIN="${GZMO_BIN:-}"
if [[ -n "$BIN" ]]; then
  gzmo_has_session_close "$BIN" || {
    echo "[!] GZMO_BIN=$BIN does not implement session close --takeaway" >&2
    exit 1
  }
else
  # Prefer temp-bench / CARGO_TARGET — repo target/release can be a REPL-only stub.
  for cand in \
    "$HOME/github-clone/temp-bench/target/release/gzmo" \
    "${CARGO_TARGET_DIR:-}/release/gzmo" \
    "$ROOT/target/release/gzmo"; do
    [[ -n "$cand" ]] || continue
    if gzmo_has_session_close "$cand"; then
      BIN="$cand"
      break
    fi
  done
fi
CFG="${GZMO_CONFIG:-$ROOT/config/gzmo.toml}"
mkdir -p "$OUT" "$SESSIONS" "$QUEUE"

if ! command -v herdr >/dev/null 2>&1; then
  echo "[!] herdr not on PATH" >&2
  exit 1
fi
[[ -x "${BIN:-}" ]] || { echo "[!] no gzmo binary with working 'session close --takeaway'" >&2; exit 1; }
[[ -f "$CFG" ]] || { echo "[!] missing $CFG" >&2; exit 1; }

bash "$ROOT/scripts/herdr-metabolism-link.sh"

MARKER="HerdrTakeaway-$(date -u +%Y%m%dT%H%M%SZ)-$$"
SID="herdr-lab-$$"
NOW="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
cat >"$SESSIONS/${SID}.json" <<EOF
{
  "id": "${SID}",
  "name": "herdr_metabolism_lab",
  "created_at": "${NOW}",
  "last_active_at": "${NOW}",
  "messages": [
    {"role": "user", "content": "Herdr close-ritual lab session.", "is_meta": false},
    {"role": "assistant", "content": "Ready for takeaway.", "is_meta": false}
  ]
}
EOF

export GZMO_BIN="$BIN"
export GZMO_CONFIG="$CFG"
export GZMO_ALLOW_LAB_VAULT=1
export GZMO_INSTANCE="${GZMO_INSTANCE:-next}"
export GZMO_SESSION_ID="$SID"
export TAKEAWAY="$MARKER: herdr session-close feeds distill queue (lab, no --now)"

CLOSE_LOG="$OUT/close-ritual.log"
set +e
bash "$PLUGIN/scripts/session-close.sh" --session "$SID" --takeaway "$TAKEAWAY" \
  >"$CLOSE_LOG" 2>&1
close_rc=$?
set -e

# Also prove via takeaway-ritual pattern evidence
export OUT SID MARKER SESSIONS QUEUE close_rc CLOSE_LOG
python3 - <<'PY'
import json, os, re
from datetime import datetime, timezone
from pathlib import Path

out = Path(os.environ["OUT"])
sid = os.environ["SID"]
marker = os.environ["MARKER"]
sessions = Path(os.environ["SESSIONS"])
queue = Path(os.environ["QUEUE"])
close_rc = int(os.environ.get("close_rc", "1"))

session_path = sessions / f"{sid}.json"
session_ok = False
if session_path.is_file():
    text = session_path.read_text(encoding="utf-8", errors="replace")
    session_ok = "[TAKEAWAY]" in text and marker.split(":")[0] in text

queued = False
queue_hits = []
if queue.is_dir():
    for p in queue.rglob("*"):
        if p.is_file():
            try:
                t = p.read_text(encoding="utf-8", errors="replace")
            except Exception:
                continue
            if marker.split(":")[0] in t or sid in t:
                queued = True
                queue_hits.append(str(p))

# Redis distill queue soft (optional; worker may BRPOP before we LLEN)
redis_depth = None
try:
    import subprocess
    r = subprocess.run(
        ["redis-cli", "-h", "127.0.0.1", "-p", "6379", "LLEN", "gzmo-next:distill:pending"],
        capture_output=True, text=True, timeout=3,
    )
    if r.returncode == 0 and r.stdout.strip().isdigit():
        redis_depth = int(r.stdout.strip())
except Exception:
    pass

close_log = Path(os.environ.get("CLOSE_LOG", out / "close-ritual.log"))
log_enqueued = False
if close_log.is_file():
    log_text = close_log.read_text(encoding="utf-8", errors="replace")
    log_enqueued = "distill job enqueued" in log_text.lower() or "distill job enqueued" in log_text

# Session TAKEAWAY + successful close is the demable bar; queue may already be drained.
ok = close_rc == 0 and session_ok

payload = {
    "schema": "gzmo.unpark.herdr.close_ritual/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "ok": ok,
    "close_rc": close_rc,
    "session_id": sid,
    "marker": marker,
    "session_has_takeaway": session_ok,
    "queue_file_hit": queued,
    "queue_hits": queue_hits[:5],
    "redis_depth_gzmo_next": redis_depth,
    "log_enqueued": log_enqueued,
    "now_flag": False,
    "advice": "herdr_close_ritual_ok" if ok else "herdr_close_ritual_fail",
}
(out / "close-ritual.json").write_text(json.dumps(payload, indent=2) + "\n")
print(json.dumps(payload, indent=2))
raise SystemExit(0 if ok else 1)
PY

bash "$PLUGIN/scripts/status.sh" >"$OUT/status.txt" 2>&1 || true
bash "$ROOT/scripts/herdr-metabolism-check.sh"
echo "[OK] herdr metabolism close-ritual demo → $OUT/close-ritual.json"
