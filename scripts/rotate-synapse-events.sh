#!/usr/bin/env bash
# Rotate append-only Synapse bus when events.jsonl exceeds a line threshold.
# Archives with gzip, truncates active file, resets pi-reader byte offset.
#
# Usage: rotate-synapse-events.sh [--dry-run] [--max-lines N] [--gzmo-root PATH]
set -euo pipefail

DRY_RUN=0
MAX_LINES="${SYNAPSE_ROTATE_MAX_LINES:-500000}"
GZMO_ROOT="${GZMO_ROOT:-/opt/gzmo/survey_GZMO}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run) DRY_RUN=1; shift ;;
    --max-lines) MAX_LINES="$2"; shift 2 ;;
    --gzmo-root) GZMO_ROOT="$2"; shift 2 ;;
    *) echo "unknown arg: $1" >&2; exit 2 ;;
  esac
done

EVENTS="${SYNAPSE_EVENTS_FILE:-$GZMO_ROOT/data/Synapse/events.jsonl}"
READER_STATE="${SYNAPSE_READER_STATE:-$GZMO_ROOT/data/synapse-reader.state.json}"
ARCHIVE_DIR="$GZMO_ROOT/data/Synapse/archive"
LOCK_FILE="${EVENTS}.rotate.lock"

if [[ ! -f "$EVENTS" ]]; then
  echo "rotate-synapse: no events file at $EVENTS (skip)"
  exit 0
fi

line_count="$(wc -l < "$EVENTS" | tr -d ' ')"
if [[ "$line_count" -le "$MAX_LINES" ]]; then
  echo "rotate-synapse: lines=$line_count threshold=$MAX_LINES (ok)"
  exit 0
fi

stamp="$(date -u +"%Y-%m-%dT%H-%M-%SZ")"
archive="$ARCHIVE_DIR/events-${stamp}.jsonl.gz"

echo "rotate-synapse: lines=$line_count exceeds $MAX_LINES → $archive"

if [[ "$DRY_RUN" -eq 1 ]]; then
  echo "rotate-synapse: dry-run only"
  exit 0
fi

mkdir -p "$ARCHIVE_DIR"
exec 9>"$LOCK_FILE"
if ! flock -n 9; then
  echo "rotate-synapse: another rotation in progress" >&2
  exit 3
fi

gzip -c "$EVENTS" > "$archive"
: > "$EVENTS"

if [[ -f "$READER_STATE" ]]; then
  python3 - <<PY
import json, pathlib
p = pathlib.Path("$READER_STATE")
try:
    state = json.loads(p.read_text())
except Exception:
    state = {}
state["byte_offset"] = 0
state["rotated_at"] = "$stamp"
p.write_text(json.dumps(state, indent=2) + "\n")
PY
fi

echo "rotate-synapse: archived $(du -h "$archive" | awk '{print $1}'), truncated active bus"
