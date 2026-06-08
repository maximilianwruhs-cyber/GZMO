#!/usr/bin/env bash
# Architecture-first migration orchestrator:
#   1. Pre-flight checks
#   2. Live ingest docs/GZMO_SYSTEM_ARCHITECTURE_INGEST.md (skip if OK in progress)
#   3. slow-reingest-migration.sh for 33 curated wave-1 files
#
# Usage:
#   ./scripts/run-migration-ingest.sh --dry-run
#   ./scripts/run-migration-ingest.sh --interval 300
#   ./scripts/run-migration-ingest.sh --skip-architecture   # wave-1 only
#   ./scripts/run-migration-ingest.sh --architecture-only   # step 0 only
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

INTERVAL=300
DRY_RUN=0
SKIP_ARCH=0
ARCH_ONLY=0
EXTRA_MIGRATION_ARGS=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run) DRY_RUN=1 ;;
    --interval) INTERVAL="${2:-300}"; shift ;;
    --skip-architecture) SKIP_ARCH=1 ;;
    --architecture-only) ARCH_ONLY=1 ;;
    --start) START="${2:-0}"; shift; EXTRA_MIGRATION_ARGS+=(--start "$START") ;;
    --max) MAX="${2:-0}"; shift; EXTRA_MIGRATION_ARGS+=(--max "$MAX") ;;
    *) echo "Unknown arg: $1" >&2; exit 1 ;;
  esac
  shift
done

ARCH_DOC="$ROOT/docs/GZMO_SYSTEM_ARCHITECTURE_INGEST.md"
PROGRESS="$ROOT/logs/migration-ingest-progress.txt"
ARCH_KEY="GZMO_SYSTEM_ARCHITECTURE_INGEST.md"

mkdir -p "$ROOT/logs"

echo "=== run-migration-ingest $(date -Is) ==="

if [[ "$DRY_RUN" -eq 0 ]]; then
  if pgrep -f '/target/release/gzmo daemon' >/dev/null 2>&1; then
    echo "[!] Stop gzmo daemon first: pkill -TERM -f 'target/release/gzmo daemon'" >&2
    exit 1
  fi
  prime_code="$(curl -s -o /dev/null -w '%{http_code}' http://localhost:8000/v1/models 2>/dev/null || echo 000)"
  if [[ "$prime_code" != "200" ]]; then
    echo "[!] Prime not up (HTTP $prime_code)" >&2
    exit 1
  fi
  echo "[PASS] pre-flight: daemon stopped, Prime :8000 OK"
  ./scripts/memory-status.sh
else
  echo "[DRY RUN] pre-flight skipped"
fi

arch_done=0
if [[ -f "$PROGRESS" ]] && grep -q "^OK ${ARCH_KEY}$" "$PROGRESS" 2>/dev/null; then
  arch_done=1
  echo "[*] architecture doc already ingested (progress OK)"
fi

if [[ "$SKIP_ARCH" -eq 0 && "$arch_done" -eq 0 ]]; then
  if [[ ! -f "$ARCH_DOC" ]]; then
    echo "[!] missing: $ARCH_DOC" >&2
    exit 2
  fi
  if [[ "$DRY_RUN" -eq 1 ]]; then
    echo "[DRY RUN] would ingest: $ARCH_DOC"
  else
    echo "[*] Step 0: ingesting architecture overview..."
    unset CARGO_TARGET_DIR
    cargo build --release -p gzmo-cli -q
    RUST_LOG=warn "$ROOT/target/release/gzmo" ingest "$ARCH_DOC"
    echo "OK $ARCH_KEY" >>"$PROGRESS"
    ./scripts/memory-status.sh
  fi
fi

if [[ "$ARCH_ONLY" -eq 1 ]]; then
  echo "=== architecture-only complete ==="
  exit 0
fi

if [[ "$DRY_RUN" -eq 1 ]]; then
  exec "$ROOT/scripts/slow-reingest-migration.sh" --dry-run --interval "$INTERVAL"
else
  exec "$ROOT/scripts/slow-reingest-migration.sh" --interval "$INTERVAL" "${EXTRA_MIGRATION_ARGS[@]}"
fi
