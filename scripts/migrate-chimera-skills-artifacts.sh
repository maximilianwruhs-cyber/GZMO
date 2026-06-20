#!/usr/bin/env bash
set -euo pipefail

# Default paths
GZMO_ROOT="${GZMO_ROOT:-$HOME/Projects/_foundation-audit/survey_GZMO}"
GZMO_SKILLS_ROOT="${GZMO_SKILLS_ROOT:-$HOME/gzmo_skills}"

DRY_RUN=0
if [[ "${1:-}" == "--dry-run" ]]; then
  DRY_RUN=1
  echo "=== Running in DRY-RUN mode ==="
fi

SRC_DIR="$GZMO_ROOT/gzmo_skills"
DST_DIR="$GZMO_SKILLS_ROOT"

if [[ ! -d "$SRC_DIR" ]]; then
  echo "Source directory $SRC_DIR does not exist. Nothing to migrate."
  exit 0
fi

echo "Migrating Chimera files from $SRC_DIR to $DST_DIR..."

# Find all files in SRC_DIR
while IFS= read -r -d '' file; do
  rel_path="${file#$SRC_DIR/}"
  target_file="$DST_DIR/$rel_path"
  target_dir="$(dirname "$target_file")"

  echo "Checking file: $rel_path"
  
  # Check if target exists
  copied=0
  if [[ -f "$target_file" ]]; then
    # Target exists, compare mtime
    src_mtime=$(stat -c %Y "$file")
    dst_mtime=$(stat -c %Y "$target_file")

    if [[ "$src_mtime" -gt "$dst_mtime" ]]; then
      echo "  -> Target exists but is older. Will copy."
      if [[ $DRY_RUN -eq 0 ]]; then
        mkdir -p "$target_dir"
        cp -p "$file" "$target_file"
        copied=1
      else
        echo "  [DRY-RUN] Would copy $file to $target_file"
      fi
    elif [[ "$src_mtime" -eq "$dst_mtime" ]]; then
      echo "  -> Identical mtime. Skipping copy, will remove source."
      copied=1
    else
      echo "  [WARNING] Target is newer: $target_file (Source: $(date -d @"$src_mtime" -u), Target: $(date -d @"$dst_mtime" -u))"
      echo "  -> Conflict! Target is newer. Skipping to avoid overwriting newer data."
    fi
  else
    echo "  -> Target does not exist. Will copy."
    if [[ $DRY_RUN -eq 0 ]]; then
      mkdir -p "$target_dir"
      cp -p "$file" "$target_file"
      copied=1
    else
      echo "  [DRY-RUN] Would copy $file to $target_file"
    fi
  fi

  if [[ $copied -eq 1 && $DRY_RUN -eq 0 ]]; then
    rm -f "$file"
  fi
done < <(find "$SRC_DIR" -type f -print0)

# Check if SRC_DIR is empty (or has no files) and can be removed
if [[ $DRY_RUN -eq 0 ]]; then
  remaining_files=$(find "$SRC_DIR" -type f | wc -l)
  if [[ "$remaining_files" -eq 0 ]]; then
    echo "No files remaining in $SRC_DIR. Removing the Chimera folder..."
    rm -rf "$SRC_DIR"
  else
    echo "[WARNING] $remaining_files file(s) remain in $SRC_DIR. Not removing the directory."
  fi
else
  echo "[DRY-RUN] Would remove $SRC_DIR if empty."
fi

echo "Migration process finished."
