#!/usr/bin/env bash
# Gradual live ingest from wave-1 corpus (writes vault + honeypot + evidence + Neo4j).
# Start with core golden files, then expand. Prime must be up.
#
# Usage:
#   ./scripts/slow-reingest-wave.sh --dry-run
#   ./scripts/slow-reingest-wave.sh --batch 3 --sleep 60
#   CORE_ONLY=1 ./scripts/slow-reingest-wave.sh --batch 5
#   ELIGIBLE_ONLY=1 ./scripts/slow-reingest-wave.sh   # skip honeypot-excluded paths
#
# Env:
#   GZMO_WAVE1_CORPUS  — archive dir (default: ~/Schreibtisch/knowledge/archive/gzmo_obolus)
#   MANIFEST           — file list (absolute paths, one per line)
#   CORE_ONLY=1        — use scripts/ingest-quality/core-golden-files.txt (15 files)
#   ELIGIBLE_ONLY=1    — skip Sources/Chat_History/Quelltext/chat_session paths
#   QDRANT_SYNC=1      — run sync-vault-to-qdrant.sh after each file (default 0; nightly cron OK)
#   SKIP_BUILD=1       — skip cargo build
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

CORPUS="${GZMO_WAVE1_CORPUS:-$HOME/Schreibtisch/knowledge/archive/gzmo_obolus}"
IQ="$ROOT/scripts/ingest-quality"
MANIFEST="${MANIFEST:-$IQ/wave1-ingest-ready.manifest}"
CORE_LIST="$IQ/core-golden-files.txt"
BATCH=1
SLEEP_SECS=30
DRY_RUN=0
START_AT=0
MAX_FILES=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run) DRY_RUN=1 ;;
    --batch) BATCH="${2:-1}"; shift ;;
    --sleep) SLEEP_SECS="${2:-30}"; shift ;;
    --start) START_AT="${2:-0}"; shift ;;
    --max) MAX_FILES="${2:-0}"; shift ;;
    --manifest) MANIFEST="${2:-}"; shift ;;
    *) echo "Unknown arg: $1" >&2; exit 1 ;;
  esac
  shift
done

LOG="$ROOT/logs/slow-reingest-$(date +%Y%m%d-%H%M%S).log"
mkdir -p "$ROOT/logs"
PROGRESS="$ROOT/logs/slow-reingest-progress.txt"

is_eligible() {
  local base="$1"
  [[ "$base" != *Sources* && "$base" != *Chat_History* && "$base" != *Quelltext* && "$base" != *chat_session* ]]
}

# Build work queue
declare -a QUEUE=()
if [[ "${CORE_ONLY:-0}" == "1" ]]; then
  while IFS= read -r f || [[ -n "$f" ]]; do
    [[ -z "$f" || "$f" =~ ^# ]] && continue
    src="$CORPUS/$f"
    [[ -f "$src" ]] || { echo "[!] missing core file: $src" >&2; continue; }
    if [[ "${ELIGIBLE_ONLY:-0}" == "1" ]] && ! is_eligible "$f"; then
      echo "[skip] honeypot-excluded: $f" | tee -a "$LOG"
      continue
    fi
    QUEUE+=("$src")
  done <"$CORE_LIST"
else
  while IFS= read -r path || [[ -n "$path" ]]; do
    [[ -z "$path" || "$path" =~ ^# ]] && continue
    [[ -f "$path" ]] || { echo "[!] missing: $path" >&2; continue; }
    base="$(basename "$path")"
    if [[ "${ELIGIBLE_ONLY:-0}" == "1" ]] && ! is_eligible "$base"; then
      echo "[skip] honeypot-excluded: $base" | tee -a "$LOG"
      continue
    fi
    QUEUE+=("$path")
  done <"$MANIFEST"
fi

total="${#QUEUE[@]}"
if [[ "$total" -eq 0 ]]; then
  echo "[!] No files in queue" >&2
  exit 2
fi

echo "=== slow-reingest-wave $(date -Is) ===" | tee "$LOG"
echo "corpus=$CORPUS queue=$total batch=$BATCH sleep=${SLEEP_SECS}s dry_run=$DRY_RUN" | tee -a "$LOG"
echo "log=$LOG" | tee -a "$LOG"

if [[ "$DRY_RUN" -eq 1 ]]; then
  idx=0
  for path in "${QUEUE[@]}"; do
    echo "  [$idx] $(basename "$path")" | tee -a "$LOG"
    idx=$((idx + 1))
  done
  echo "=== DRY RUN — would ingest $total files ==="
  exit 0
fi

if [[ "${SKIP_BUILD:-0}" != "1" ]]; then
  unset CARGO_TARGET_DIR
  cargo build --release -p gzmo-cli -q
fi

GZMO="$ROOT/target/release/gzmo"
ok=0
fail=0
idx=0
for path in "${QUEUE[@]}"; do
  [[ "$idx" -lt "$START_AT" ]] && { idx=$((idx + 1)); continue; }
  [[ "$MAX_FILES" -gt 0 && "$((idx - START_AT))" -ge "$MAX_FILES" ]] && break

  base="$(basename "$path")"
  echo "" | tee -a "$LOG"
  echo "--- [$idx/$total] ingest: $base $(date -Is)" | tee -a "$LOG"

  if RUST_LOG=warn "$GZMO" ingest "$path" 2>&1 | tee -a "$LOG"; then
    ok=$((ok + 1))
    echo "OK $base" >>"$PROGRESS"
  else
    fail=$((fail + 1))
    echo "FAIL $base" >>"$PROGRESS"
    echo "[!] ingest failed: $base (continuing)" | tee -a "$LOG"
  fi

  # Light status every file
  "$ROOT/scripts/memory-status.sh" 2>/dev/null | tee -a "$LOG" || true

  if [[ "${QDRANT_SYNC:-0}" == "1" && -x "$ROOT/scripts/sync-vault-to-qdrant.sh" ]]; then
    "$ROOT/scripts/sync-vault-to-qdrant.sh" 2>&1 | tail -3 | tee -a "$LOG" || true
  fi

  idx=$((idx + 1))
  # Sleep between batches (not after last file)
  if [[ "$((idx % BATCH))" -eq 0 && "$idx" -lt "$total" ]]; then
    echo "[*] batch pause ${SLEEP_SECS}s..." | tee -a "$LOG"
    sleep "$SLEEP_SECS"
  fi
done

echo "" | tee -a "$LOG"
echo "=== slow-reingest done: ok=$ok fail=$fail ===" | tee -a "$LOG"
echo "progress: $PROGRESS"
echo "Next eval (after meaningful batch):"
echo "  scripts/ingest-quality/run-recall-eval.py --match strict"
echo "  scripts/ingest-quality/faithfulness-judge.py --gate"

[[ "$fail" -eq 0 ]]
