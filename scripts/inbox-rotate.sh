#!/usr/bin/env bash
set -euo pipefail
shopt -s nullglob

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$REPO_ROOT/data-next}"

MODE="dry-run"
OLDER_THAN=30
DIRS=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run) MODE="dry-run"; shift ;;
    --apply)   MODE="apply";   shift ;;
    --older-than) OLDER_THAN="$2"; shift 2 ;;
    *) DIRS+=("$1"); shift ;;
  esac
done

if [[ ${#DIRS[@]} -eq 0 ]]; then
  for d in "$DATA/inbox" "$DATA/tinyfolder-inbox"; do
    [[ -d "$d" ]] && DIRS+=("$d")
  done
fi

for dir in "${DIRS[@]}"; do
  [[ -d "$dir" ]] || continue
  candidates=()
  for f in "$dir"/drop-*.md; do
    [[ -f "$f" ]] || continue
    if [[ "$(find "$f" -maxdepth 0 -mtime +"$OLDER_THAN" 2>/dev/null)" ]]; then
      candidates+=("$f")
    fi
  done

  n=${#candidates[@]}

  if [[ "$MODE" == "dry-run" ]]; then
    echo "[rotate] would move $n file(s) from $dir (older than ${OLDER_THAN}d)"
    for f in "${candidates[@]}"; do
      echo "  $f"
    done
  else
    for f in "${candidates[@]}"; do
      if [[ "$(uname)" == "Darwin" ]]; then
        month=$(stat -f '%Sm' -t '%Y-%m' "$f")
      else
        month=$(date -r "$f" '+%Y-%m')
      fi
      dest="$dir/processed/$month"
      mkdir -p "$dest"
      mv "$f" "$dest/"
    done
    echo "[rotate] moved $n file(s) from $dir"
  fi
done

echo "[rotate] done"
