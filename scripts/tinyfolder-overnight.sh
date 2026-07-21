#!/usr/bin/env bash
# Overnight TinyFolder → living distill enqueue (no CLI chat).
# Turns the overnight spark thesis into an operator organ:
#   drop-folder notes reach distill/ingest without the chat surface.
#
# Workstation (SSH apply, dual-writer-safe):
#   bash scripts/tinyfolder-overnight.sh
#   bash scripts/tinyfolder-overnight.sh --dry-run
#
# On CT101 (systemd timer / cron — no SSH):
#   bash scripts/tinyfolder-overnight.sh --on-host
#
# Never starts gzmo-serve. Never uses --now.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
HOST="${CT101_SSH_HOST:-ct101}"
GZMO_BIN="${CT101_GZMO_BIN:-/opt/gzmo/current/target/release/gzmo}"
ON_HOST=0
DRY=0
for a in "$@"; do
  case "$a" in
    --on-host) ON_HOST=1 ;;
    --dry-run) DRY=1 ;;
  esac
done

OUT_DIR="${GZMO_DATA_NEXT:-$ROOT/data-next}/tinyfolder"
mkdir -p "$OUT_DIR"
STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
REPORT="$OUT_DIR/overnight-$STAMP.json"

if [[ "$ON_HOST" == "1" ]]; then
  INBOX="${TINYFOLDER_INBOX:-/opt/gzmo/data/inbox}"
  OUT_DIR="${GZMO_DATA_NEXT:-/opt/gzmo/data}/tinyfolder"
  mkdir -p "$INBOX/processed" "$OUT_DIR"
  REPORT="$OUT_DIR/overnight-$STAMP.json"
  mapfile -t PENDING < <(find "$INBOX" -maxdepth 1 -type f -name '*.md' ! -name 'README.md' ! -name '_*' 2>/dev/null | sort || true)
  N="${#PENDING[@]}"
  if [[ "$N" -eq 0 ]]; then
    python3 - <<PY | tee "$REPORT"
import json
print(json.dumps({
  "ok": True,
  "advice": "tinyfolder_overnight_idle — no pending drops",
  "pending": 0,
  "path": "$REPORT",
}, indent=2))
PY
    exit 0
  fi
  TAKE=$(( N < 3 ? N : 3 ))
  FILES=("${PENDING[@]:0:$TAKE}")
  if [[ "$DRY" == "1" ]]; then
    python3 - <<PY | tee "$REPORT"
import json
print(json.dumps({
  "ok": True,
  "advice": "tinyfolder_overnight_dry — would enqueue",
  "pending": $N,
  "files": $(printf '%s\n' "${FILES[@]}" | python3 -c 'import json,sys; print(json.dumps([l.strip() for l in sys.stdin if l.strip()]))'),
}, indent=2))
PY
    exit 0
  fi
  export FILES_JSON
  FILES_JSON="$(printf '%s\n' "${FILES[@]}" | python3 -c 'import json,sys; print(json.dumps([l.strip() for l in sys.stdin if l.strip()]))')"
  SID="tinyfolder-overnight-$(date -u +%s)"
  SESS="/opt/gzmo/data/sessions/${SID}.json"
  COMBINED="$(python3 - <<'PY'
import json, os
from pathlib import Path
files = json.loads(os.environ["FILES_JSON"])
lines = []
for f in files:
    t = Path(f).read_text(encoding="utf-8", errors="replace")
    if t.lstrip().startswith("---"):
        parts = t.split("---", 2)
        t = parts[2] if len(parts) > 2 else t
    line = " ".join(t.strip().split())[:280]
    if line:
        lines.append(f"TinyFolderDrop: {line}")
print(" | ".join(lines[:3]))
PY
)"
  python3 - <<PY
import json
from datetime import datetime, timezone
now = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
open("$SESS", "w").write(json.dumps({
  "id": "$SID",
  "name": "tinyfolder_overnight",
  "created_at": now,
  "last_active_at": now,
  "messages": [
    {"role": "user", "content": "TinyFolder overnight enqueue.", "is_meta": False},
    {"role": "assistant", "content": "Recording drop-folder notes as takeaway.", "is_meta": False},
  ],
}))
PY
  (cd /opt/gzmo && GZMO_CONFIG=/opt/gzmo/gzmo.toml "$GZMO_BIN" session close "$SID" --takeaway "$COMBINED")
  for f in "${FILES[@]}"; do
    mv "$f" "$INBOX/processed/" 2>/dev/null || true
  done
  python3 - <<PY | tee "$REPORT"
import json
print(json.dumps({
  "ok": True,
  "advice": "tinyfolder_overnight_enqueued",
  "session_id": "$SID",
  "moved": $TAKE,
  "path": "$REPORT",
}, indent=2))
PY
  exit 0
fi

# Workstation path: refuse dual-writer, then scan+apply via existing drop script.
if systemctl --user is-active gzmo-serve.service 2>/dev/null | grep -qx active; then
  echo "[!] refused_dual_writer — stop gzmo-serve before overnight TinyFolder apply" >&2
  exit 2
fi

if [[ "$DRY" == "1" ]]; then
  bash "$ROOT/scripts/tinyfolder-drop.sh" --scan --living
  echo "[OK] dry-run — living-enqueue.json written (no apply)"
  exit 0
fi

bash "$ROOT/scripts/tinyfolder-drop.sh" --scan --living --apply-takeaway
cp -f "${GZMO_DATA_NEXT:-$ROOT/data-next}/tinyfolder/living-enqueue.json" "$REPORT" 2>/dev/null || true
echo "[OK] tinyfolder overnight apply → $REPORT (host=$HOST)"
echo "On CT101 timer: bash scripts/tinyfolder-overnight.sh --on-host"
