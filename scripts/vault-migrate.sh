#!/usr/bin/env bash
# Offline CT101 → GZMO-next vault migrate helper (stretch S3).
# Default is --dry-run. Fresh data-next vault remains valid; apply is operator-explicit.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CLONE="${GZMO_CLONE_ROOT:-$(cd "$ROOT/.." && pwd)}"
NEXT_VAULT="${NEXT_VAULT:-$ROOT/data-next/vault.db}"
SRC_VAULT="${SRC_VAULT:-/opt/gzmo/data/vault.db}"
SCHEDULER_PID_FILE="${GZMO_SCHEDULER_PID:-/tmp/gzmo-scheduler.pid}"
MODE="dry-run"
YES=0

usage() {
  cat <<EOF
Usage: $0 [--dry-run|--apply] [--src PATH] [--dest PATH] [--yes]

  --dry-run   Print freeze/backup/copy/sync/verify steps; refuse if scheduler lock present (default)
  --apply     Perform backup + copy (+ optional Qdrant full sync). Requires --yes
  --src PATH  Source vault (default: $SRC_VAULT)
  --dest PATH Destination vault (default: $NEXT_VAULT)
  --yes       Confirm destructive apply

Anti-goals: no online CT101↔next sync; no silent overwrite without .bak.
EOF
  exit 1
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run) MODE="dry-run"; shift ;;
    --apply) MODE="apply"; shift ;;
    --src) SRC_VAULT="$2"; shift 2 ;;
    --dest) NEXT_VAULT="$2"; shift 2 ;;
    --yes) YES=1; shift ;;
    -h|--help) usage ;;
    *) echo "Unknown: $1"; usage ;;
  esac
done

refuse_if_scheduler_running() {
  if [[ -f "$SCHEDULER_PID_FILE" ]]; then
    local pid
    pid="$(cat "$SCHEDULER_PID_FILE" 2>/dev/null || true)"
    if [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null; then
      echo "REFUSE: gzmo-scheduler appears running (pid=$pid lock=$SCHEDULER_PID_FILE)" >&2
      echo "Stop the scheduler before migrate." >&2
      return 2
    fi
  fi
  return 0
}

print_steps() {
  cat <<EOF
=== vault-migrate ($MODE) ===
Checklist:
  1. Freeze CT101 consumers (systemctl stop gzmo-daemon on CT101; snapshot LXC/volume)
  2. Stop gzmo-scheduler on workstation (and any gzmo daemon next)
  3. Backup dest: ${NEXT_VAULT}.bak-\$(date -u +%Y%m%dT%H%M%SZ)
  4. Copy src → dest (include -wal/-shm if hot copy): 
       src=$SRC_VAULT
       dest=$NEXT_VAULT
  5. Full Qdrant rebuild: bash $ROOT/scripts/qdrant-vault-sync.sh
       then bash $ROOT/scripts/qdrant-post-sync-verify.sh (if present)
  6. Beat-gates: bash $CLONE/little-tools-lab/scripts/ci/beat-gates-fixture.sh
       then live discovery beat: beat-gate.sh --loop discovery --live
  7. Restart scheduler; watch data-next/scheduler-runs/latest.json

Decision gate: Import CT101 vault only if product needs historical depth.
Fresh organic growth of data-next remains valid.
EOF
}

sched_rc=0
refuse_if_scheduler_running || sched_rc=$?
print_steps

if [[ "$MODE" == "dry-run" ]]; then
  if [[ "$sched_rc" -ne 0 ]]; then
    echo
    echo "DRY-RUN NOTE: scheduler lock present — apply would refuse until stopped."
  fi
  echo
  echo "DRY-RUN only — no files written."
  if [[ -f "$SRC_VAULT" && -f "$NEXT_VAULT" ]]; then
    echo
    echo "=== vault-diff preview ==="
    python3 "$ROOT/scripts/vault-diff.py" --left "$NEXT_VAULT" --right "$SRC_VAULT" || true
  else
    echo "NOTE: skipping vault-diff preview (src or dest missing on this host)."
    echo "  src_exists=$([[ -f $SRC_VAULT ]] && echo yes || echo no) dest_exists=$([[ -f $NEXT_VAULT ]] && echo yes || echo no)"
  fi
  exit 0
fi

# apply path
if [[ "$sched_rc" -ne 0 ]]; then
  exit 2
fi
if [[ "$YES" -ne 1 ]]; then
  echo "REFUSE: --apply requires --yes" >&2
  exit 2
fi
[[ -f "$SRC_VAULT" ]] || { echo "FAIL: src missing: $SRC_VAULT" >&2; exit 2; }
[[ -f "$NEXT_VAULT" ]] || { echo "FAIL: dest missing (will not create empty parent): $NEXT_VAULT" >&2; exit 2; }

STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
BAK="${NEXT_VAULT}.bak-${STAMP}"
echo "Backing up $NEXT_VAULT → $BAK"
cp -a "$NEXT_VAULT" "$BAK"
[[ -f "${NEXT_VAULT}-wal" ]] && cp -a "${NEXT_VAULT}-wal" "${BAK}-wal" || true
[[ -f "${NEXT_VAULT}-shm" ]] && cp -a "${NEXT_VAULT}-shm" "${BAK}-shm" || true

echo "Copying $SRC_VAULT → $NEXT_VAULT"
cp -a "$SRC_VAULT" "$NEXT_VAULT"
[[ -f "${SRC_VAULT}-wal" ]] && cp -a "${SRC_VAULT}-wal" "${NEXT_VAULT}-wal" || true
[[ -f "${SRC_VAULT}-shm" ]] && cp -a "${SRC_VAULT}-shm" "${NEXT_VAULT}-shm" || true

echo "Qdrant full sync…"
bash "$ROOT/scripts/qdrant-vault-sync.sh"
if [[ -x "$ROOT/scripts/qdrant-post-sync-verify.sh" ]]; then
  bash "$ROOT/scripts/qdrant-post-sync-verify.sh" || {
    echo "WARN: post-sync verify failed — restore from $BAK if needed" >&2
    exit 3
  }
fi

echo "APPLY complete. Backup at $BAK"
python3 "$ROOT/scripts/vault-diff.py" --left "$NEXT_VAULT" --right "$SRC_VAULT" || true
