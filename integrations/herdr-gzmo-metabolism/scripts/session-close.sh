#!/usr/bin/env bash
# Close ritual: durable takeaway → gzmo session close → distill enqueue.
set -euo pipefail
# shellcheck disable=SC1091
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib.sh"
export_gzmo_env

INTERACTIVE=0
NOW_FLAG=()
TAKEAWAY="${TAKEAWAY:-}"
SESSION_ID="${GZMO_SESSION_ID:-}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --interactive) INTERACTIVE=1; shift ;;
    --now) NOW_FLAG=(--now); shift ;;
    --takeaway) TAKEAWAY="${2:-}"; shift 2 ;;
    --session) SESSION_ID="${2:-}"; shift 2 ;;
    *) shift ;;
  esac
done

if [[ -z "$TAKEAWAY" && -n "${HERDR_PLUGIN_CONTEXT_JSON:-}" ]]; then
  TAKEAWAY="$(python3 - <<'PY'
import json, os
ctx = json.loads(os.environ.get("HERDR_PLUGIN_CONTEXT_JSON") or "{}")
# Prefer explicit selection; else leave empty for interactive/prompt.
sel = ctx.get("selected_text") or ctx.get("selectedText") or ""
print(sel.strip() if isinstance(sel, str) else "")
PY
)"
fi

if [[ -z "$TAKEAWAY" && "$INTERACTIVE" -eq 1 ]]; then
  echo "GZMO metabolism — durable takeaway for distill"
  echo "(pane=${HERDR_PANE_ID:-none} workspace=${HERDR_WORKSPACE_ID:-none})"
  echo -n "Takeaway: "
  IFS= read -r TAKEAWAY || true
fi

if [[ -z "$TAKEAWAY" ]]; then
  echo "[!] No takeaway. Set TAKEAWAY=…, select text in herdr, or use --interactive." >&2
  echo "    Example: TAKEAWAY='felt recall needs session close' herdr plugin action invoke gzmo.metabolism.session-close" >&2
  exit 1
fi

ARGS=(session close)
if [[ -n "$SESSION_ID" ]]; then
  ARGS+=("$SESSION_ID")
fi
ARGS+=(--takeaway "$TAKEAWAY")
if [[ ${#NOW_FLAG[@]} -gt 0 ]]; then
  ARGS+=("${NOW_FLAG[@]}")
fi

echo "[*] $GZMO_BIN ${ARGS[*]}"
"$GZMO_BIN" "${ARGS[@]}"

export TAKEAWAY STATE_DIR
python3 - <<'PY'
import json, os, time
state = os.environ.get("STATE_DIR", ".")
path = os.path.join(state, "session-close-latest.json")
payload = {
    "ok": True,
    "ts": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
    "takeaway": os.environ.get("TAKEAWAY", ""),
    "pane_id": os.environ.get("HERDR_PANE_ID"),
    "workspace_id": os.environ.get("HERDR_WORKSPACE_ID"),
    "gzmo_config": os.environ.get("GZMO_CONFIG"),
}
open(path, "w", encoding="utf-8").write(json.dumps(payload, indent=2) + "\n")
print(json.dumps({"ok": True, "path": path}, indent=2))
PY
