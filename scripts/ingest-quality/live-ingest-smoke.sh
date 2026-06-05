#!/usr/bin/env bash
# F18: one-file LIVE ingest smoke — exercises Neo4j MCP write path (not ingest-eval dry-run).
#
# Usage:
#   LIVE_INGEST_SMOKE=1 scripts/ingest-quality/certify-production-baseline.sh
#   scripts/ingest-quality/live-ingest-smoke.sh
#
# Env:
#   LIVE_INGEST_SMOKE_FILE  override smoke file (must be honeypot-eligible path)
#   GZMO_WAVE1_CORPUS       corpus root (default: ~/Schreibtisch/knowledge/archive/gzmo_obolus)
#   SKIP_BUILD=1            skip cargo build
#   SKIP_HEALTH=1           skip gzmo health preflight

set -eo pipefail
export LC_ALL=C

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
ROOT="$(cd "$DIR/../.." >/dev/null 2>&1 && pwd)"
cd "$ROOT"
unset CARGO_TARGET_DIR

CORPUS="${GZMO_WAVE1_CORPUS:-$HOME/Schreibtisch/knowledge/archive/gzmo_obolus}"
DEFAULT_FILE="wave_01_gzmo_obolus_drive_cleanTakeoutDriveObolusObolus-masteragentsbackup_custodianmd.md"
SMOKE_FILE="${LIVE_INGEST_SMOKE_FILE:-$DEFAULT_FILE}"
SMOKE_PATH="$CORPUS/$SMOKE_FILE"
LOG="${ROOT}/logs/live-ingest-smoke.log"
mkdir -p "${ROOT}/logs"

exec > >(tee -a "$LOG") 2>&1
echo "=== Live ingest smoke (Neo4j MCP write path) ==="
echo "started: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo "file:    $SMOKE_PATH"

FAIL=0

if [[ "$SKIP_BUILD" != "1" ]]; then
  if cargo build --release -p gzmo-cli -q 2>/dev/null; then
    echo "[PASS] cargo build --release -p gzmo-cli"
  else
    echo "[FAIL] cargo build"
    exit 1
  fi
fi

if [[ ! -f "$SMOKE_PATH" ]]; then
  echo "[FAIL] smoke file missing: $SMOKE_PATH"
  echo "  set LIVE_INGEST_SMOKE_FILE or GZMO_WAVE1_CORPUS to a honeypot-eligible golden file"
  exit 1
fi

if [[ "$SKIP_HEALTH" != "1" ]]; then
  if ./target/release/gzmo health 2>&1 | tee -a "$LOG"; then
    echo "[PASS] gzmo health (MCP memory + infra)"
  else
    echo "[FAIL] gzmo health — Neo4j MCP or Prime/embed/rerank unavailable"
    exit 1
  fi
fi

echo ""
echo "[*] live ingest (writes vault + Neo4j via MCP)..."
INGEST_OUT="$(mktemp)"
set +e
RUST_LOG=warn ./target/release/gzmo ingest "$SMOKE_PATH" 2>&1 | tee "$INGEST_OUT"
INGEST_RC=${PIPESTATUS[0]}
set -e

if [[ "$INGEST_RC" -ne 0 ]]; then
  echo "[FAIL] gzmo ingest exited $INGEST_RC"
  rm -f "$INGEST_OUT"
  exit 1
fi

KG_LINE="$(grep -E '^KG: [0-9]+ entities, [0-9]+ relations' "$INGEST_OUT" | tail -1 || true)"
if [[ -z "$KG_LINE" ]]; then
  echo "[FAIL] ingest output missing KG line (Neo4j write path not confirmed)"
  FAIL=1
else
  echo "[ok] $KG_LINE"
  ENTITIES="$(echo "$KG_LINE" | sed -n 's/^KG: \([0-9]*\) entities.*/\1/p')"
  RELATIONS="$(echo "$KG_LINE" | sed -n 's/^KG: [0-9]* entities, \([0-9]*\) relations.*/\1/p')"
  if [[ "${ENTITIES:-0}" -eq 0 && "${RELATIONS:-0}" -eq 0 ]]; then
    echo "[FAIL] Neo4j MCP wrote 0 entities and 0 relations (strict_kg should have aborted)"
    FAIL=1
  else
    echo "[PASS] Neo4j MCP write path exercised (entities=$ENTITIES relations=$RELATIONS)"
  fi
fi

VAULT_DB="${ROOT}/data/vault.db"
if [[ -f "$VAULT_DB" ]]; then
  LEAF="$(basename "$SMOKE_FILE")"
  JOINED="$(python3 -c "
import sqlite3
c = sqlite3.connect('$VAULT_DB')
n = c.execute('''
  SELECT COUNT(*)
  FROM honeypot h
  JOIN evidence e ON e.fact_id = h.id
  WHERE h.is_latest = 1 AND lower(h.source_file) LIKE ?
''', ('%${LEAF,,}%',)).fetchone()[0]
print(n)
" 2>/dev/null || echo 0)"
  if [[ "${JOINED:-0}" -gt 0 ]]; then
    echo "[PASS] vault evidence join rows for smoke file: $JOINED"
  else
    echo "[WARN] no evidence join rows for $LEAF (re-ingest/backfill may be pending)"
  fi
fi

rm -f "$INGEST_OUT"

if [[ "$FAIL" -ne 0 ]]; then
  echo ""
  echo "RESULT: LIVE INGEST SMOKE FAILED (see $LOG)"
  exit 1
fi

echo ""
echo "RESULT: LIVE INGEST SMOKE PASSED"
echo "finished: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
exit 0
