#!/usr/bin/env bash
# Soft reminder when a pane closes without the close ritual (never auto-distills).
set -euo pipefail
# shellcheck disable=SC1091
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib.sh"

export LOG="$STATE_DIR/missed-close.jsonl"
export PANE="${HERDR_PANE_ID:-}"
export EVT="${HERDR_PLUGIN_EVENT:-pane.closed}"

python3 - <<'PY'
import json, os, time
log = os.environ["LOG"]
row = {
    "ts": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
    "event": os.environ.get("EVT", "pane.closed"),
    "pane_id": os.environ.get("PANE") or None,
    "workspace_id": os.environ.get("HERDR_WORKSPACE_ID"),
    "hint": "Invoke gzmo.metabolism.session-close (or popup close-ritual) before closing agent panes.",
}
with open(log, "a", encoding="utf-8") as fh:
    fh.write(json.dumps(row) + "\n")
PY

# Stay quiet in herdr event hooks (exit 0 always).
exit 0
