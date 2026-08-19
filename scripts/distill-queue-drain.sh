#!/usr/bin/env bash
# Non-destructive drainer for the WS distill-queue.
# Moves aged *.jsonl files into archive/<YYYY-MM>/ (mkdir -p). Never rm.
#
# Usage: bash scripts/distill-queue-drain.sh [--dry-run|--apply] [--older-than DAYS]
# Defaults: --dry-run, --older-than 14
set -euo pipefail
shopt -s nullglob

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
QUEUE_DIR="$DATA/distill-queue"
ARCHIVE_ROOT="$QUEUE_DIR/archive"

# --- defaults ---
mode="dry-run"
older_than=14

# --- arg parse ---
while (( $# > 0 )); do
    case "$1" in
        --dry-run)  mode="dry-run"; shift ;;
        --apply)    mode="apply"; shift ;;
        --older-than)
            shift
            if (( $# == 0 )); then
                echo "[drain] error: --older-than requires a value" >&2
                exit 2
            fi
            older_than="$1"
            shift
            ;;
        -h|--help)
            sed -n '2,8p' "$0" | sed 's/^# \{0,1\}//'
            exit 0
            ;;
        *)
            echo "[drain] error: unknown argument: $1" >&2
            exit 2
            ;;
    esac
done

# --- validate older_than is a non-negative integer ---
if ! [[ "$older_than" =~ ^[0-9]+$ ]]; then
    echo "[drain] error: --older-than must be a non-negative integer, got: $older_than" >&2
    exit 2
fi

# --- queue dir must exist ---
if [[ ! -d "$QUEUE_DIR" ]]; then
    echo "[drain] queue dir not found: $QUEUE_DIR"
    echo "[drain] done"
    exit 0
fi

# --- collect candidates: *.jsonl in queue dir (not in archive/), older than N days ---
candidates=()
while IFS= read -r -d '' f; do
    candidates+=("$f")
done < <(find "$QUEUE_DIR" -maxdepth 1 -type f -name '*.jsonl' -mtime +"$older_than" -print0 2>/dev/null || true)

if (( ${#candidates[@]} == 0 )); then
    echo "[drain] would archive 0 file(s), 0 entries (older than ${older_than}d)"
    echo "[drain] done"
    exit 0
fi

# --- sum total entries across candidates ---
total_entries=0
for f in "${candidates[@]}"; do
    n="$(cat "$f" 2>/dev/null | wc -l | tr -d ' ' || printf '0')"
    total_entries=$(( total_entries + n ))
done

# --- report header ---
verb="would archive"
if [[ "$mode" == "apply" ]]; then
    verb="archived"
    mkdir -p "$ARCHIVE_ROOT"
fi
echo "[drain] $verb ${#candidates[@]} file(s), $total_entries entries (older than ${older_than}d)"

# --- per-file action ---
for f in "${candidates[@]}"; do
    n="$(cat "$f" 2>/dev/null | wc -l | tr -d ' ' || printf '0')"
    if [[ "$mode" == "apply" ]]; then
        # month of file mtime
        ym="$(date -r "$f" '+%Y-%m' 2>/dev/null || date '+%Y-%m')"
        dest="$ARCHIVE_ROOT/$ym"
        mkdir -p "$dest"
        mv "$f" "$dest/"
        echo "  - $(basename "$f"): $n entries → archive/$ym/"
    else
        echo "  - $(basename "$f"): $n entries"
    fi
done

echo "[drain] done"
exit 0
