#!/usr/bin/env bash
# Curated wave-1 live migration: one file every INTERVAL seconds (default 300 = 5 min).
# Queue: scripts/ingest-quality/wave1-migration-curated.manifest (33 honeypot-eligible files).
#
# Usage:
#   ./scripts/slow-reingest-migration.sh --dry-run
#   ./scripts/slow-reingest-migration.sh --interval 300
#   ./scripts/slow-reingest-migration.sh --interval 300 --start 5
#   QDRANT_SYNC=1 ./scripts/slow-reingest-migration.sh --interval 300
#
# Env: SKIP_BUILD=1, QDRANT_SYNC=0|1, MANIFEST=path
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

MANIFEST="${MANIFEST:-$ROOT/scripts/ingest-quality/wave1-migration-curated.manifest}"
INTERVAL=300
DRY_RUN=0
START_AT=0
MAX_FILES=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run) DRY_RUN=1 ;;
    --interval) INTERVAL="${2:-300}"; shift ;;
    --start) START_AT="${2:-0}"; shift ;;
    --max) MAX_FILES="${2:-0}"; shift ;;
    --manifest) MANIFEST="${2:-}"; shift ;;
    *) echo "Unknown arg: $1" >&2; exit 1 ;;
  esac
  shift
done

LOG="$ROOT/logs/migration-ingest-$(date +%Y%m%d-%H%M%S).log"
PROGRESS="$ROOT/logs/migration-ingest-progress.txt"
mkdir -p "$ROOT/logs"

declare -a QUEUE=()
while IFS= read -r path || [[ -n "$path" ]]; do
  [[ -z "$path" || "$path" =~ ^# ]] && continue
  [[ -f "$path" ]] || { echo "[!] missing: $path" >&2; exit 2; }
  QUEUE+=("$path")
done <"$MANIFEST"

total="${#QUEUE[@]}"
if [[ "$total" -eq 0 ]]; then
  echo "[!] empty manifest: $MANIFEST" >&2
  exit 2
fi

# Skip basenames already marked OK in progress file
declare -A DONE=()
if [[ -f "$PROGRESS" ]]; then
  while IFS= read -r line; do
    [[ "$line" =~ ^OK\  ]] && DONE["${line#OK }"]=1
  done <"$PROGRESS"
fi

echo "=== slow-reingest-migration $(date -Is) ===" | tee "$LOG"
echo "manifest=$MANIFEST queue=$total interval=${INTERVAL}s dry_run=$DRY_RUN start=$START_AT" | tee -a "$LOG"

if [[ "$DRY_RUN" -eq 1 ]]; then
  idx=0
  eta_min=$(( (total - START_AT) * INTERVAL / 60 ))
  echo "ETA sleep-only: ~${eta_min} min (+ ingest time per file)" | tee -a "$LOG"
  for path in "${QUEUE[@]}"; do
    base="$(basename "$path")"
    skip=""
    [[ -n "${DONE[$base]:-}" ]] && skip=" [SKIP OK]"
    echo "  [$idx] $base$skip" | tee -a "$LOG"
    idx=$((idx + 1))
  done
  echo "=== DRY RUN complete ==="
  exit 0
fi

if pgrep -f '/target/release/gzmo daemon' >/dev/null 2>&1; then
  echo "[!] gzmo daemon is running — stop it before migration ingest" >&2
  exit 1
fi

prime_code="$(curl -s -o /dev/null -w '%{http_code}' http://localhost:8000/v1/models 2>/dev/null || echo 000)"
if [[ "$prime_code" != "200" ]]; then
  echo "[!] Prime not reachable at :8000 (HTTP $prime_code)" >&2
  exit 1
fi

if [[ "${SKIP_BUILD:-0}" != "1" ]]; then
  unset CARGO_TARGET_DIR
  cargo build --release -p gzmo-cli -q
fi
GZMO="$ROOT/target/release/gzmo"

ok=0
fail=0
skipped=0
idx=0
for path in "${QUEUE[@]}"; do
  base="$(basename "$path")"
  if [[ -n "${DONE[$base]:-}" ]]; then
    skipped=$((skipped + 1))
    idx=$((idx + 1))
    continue
  fi
  [[ "$idx" -lt "$START_AT" ]] && { idx=$((idx + 1)); continue; }
  [[ "$MAX_FILES" -gt 0 && "$((ok + fail))" -ge "$MAX_FILES" ]] && break

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

  "$ROOT/scripts/memory-status.sh" 2>/dev/null | tee -a "$LOG" || true

  if [[ "${QDRANT_SYNC:-0}" == "1" && -x "$ROOT/scripts/sync-vault-to-qdrant.sh" ]]; then
    "$ROOT/scripts/sync-vault-to-qdrant.sh" 2>&1 | tail -3 | tee -a "$LOG" || true
  fi

  idx=$((idx + 1))
  if [[ "$idx" -lt "$total" && "$INTERVAL" -gt 0 ]]; then
    next=$((idx < total ? idx : total - 1))
    echo "[*] pause ${INTERVAL}s before next file..." | tee -a "$LOG"
    sleep "$INTERVAL"
  fi
done

echo "" | tee -a "$LOG"
echo "=== migration ingest done: ok=$ok fail=$fail skipped=$skipped ===" | tee -a "$LOG"
echo "progress: $PROGRESS"
echo "log: $LOG"
echo "Next: see docs/MIGRATION_INGEST_RUNBOOK.md Phase 4"

[[ "$fail" -eq 0 ]]
