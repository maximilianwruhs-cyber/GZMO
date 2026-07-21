#!/usr/bin/env bash
# Sync Brain Feed / keep-quality scripts+docs to CT101 without rebuild or daemon restart.
# Encodes docs/CT101_DEPLOY.md §"Sync docs/scripts only". Restores +x (rsync drops it).
#
#   bash scripts/ct101-brain-feed-sync.sh
#
# Dual-writer safe: never systemctl restart. Never touches .env / vault / compose.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
HOST="${CT101_SSH_HOST:-ct101}"
REMOTE_ROOT="${CT101_GZMO_ROOT:-/opt/gzmo/current}"

SCRIPTS=(
  brain-feed-check.sh
  brain-intel-promote.sh
  serendipity-promote.sh
  keep-quality-gate.sh
  keep-quality-soak.sh
  install-living-airgap.sh
  tinyfolder-drop.sh
  tinyfolder-check.sh
  tinyfolder-overnight.sh
  install-tinyfolder-overnight-timer.sh
  felt-use-depth.sh
  organ-trace.sh
  ct101-brain-feed-sync.sh
  opportunity-sense.sh
  opportunity-rank.sh
  opportunity-bet.sh
  opportunity-next-mission.sh
  opportunity-discovery-check.sh
  opportunity_lib.py
)

DOCS=(
  BRAIN_FEED.md
  ADR-0004-airgap-living-usp.md
  AIRGAP_LIVING.md
  KEEP_QUALITY.md
  MCP_LOCAL_ATTACH.md
  SPINE_FOCUS.md
  STACK_OPPORTUNITY_MAP.md
  UNPARK_ROADMAP.md
  CT101_DEPLOY.md
  OPPORTUNITY_DISCOVERY.md
)

ssh_ok() {
  ssh -o ConnectTimeout=12 -o BatchMode=yes "$HOST" "$@"
}

echo "[*] probe $HOST …"
ssh_ok 'true' || {
  echo "[!] SSH BatchMode failed for $HOST (set CT101_SSH_HOST?)" >&2
  exit 1
}

before_ts="$(ssh_ok 'systemctl show gzmo-daemon -p ActiveEnterTimestamp --value' 2>/dev/null || echo unknown)"

script_srcs=()
for s in "${SCRIPTS[@]}"; do
  f="$ROOT/scripts/$s"
  if [[ -f "$f" ]]; then
    script_srcs+=("$f")
  else
    echo "[!] skip missing local script: $s" >&2
  fi
done
[[ ${#script_srcs[@]} -gt 0 ]] || { echo "[!] no scripts to sync" >&2; exit 1; }

doc_srcs=()
for d in "${DOCS[@]}"; do
  f="$ROOT/docs/$d"
  if [[ -f "$f" ]]; then
    doc_srcs+=("$f")
  else
    echo "[!] skip missing local doc: $d" >&2
  fi
done

echo "[*] rsync scripts → ${HOST}:${REMOTE_ROOT}/scripts/"
ssh_ok "mkdir -p $(printf '%q' "$REMOTE_ROOT/scripts") $(printf '%q' "$REMOTE_ROOT/docs")"
rsync -az "${script_srcs[@]}" "${HOST}:${REMOTE_ROOT}/scripts/"

if [[ ${#doc_srcs[@]} -gt 0 ]]; then
  echo "[*] rsync docs → ${HOST}:${REMOTE_ROOT}/docs/"
  rsync -az "${doc_srcs[@]}" "${HOST}:${REMOTE_ROOT}/docs/"
fi

if [[ -d "$ROOT/systemd" ]]; then
  echo "[*] rsync systemd units → ${HOST}:${REMOTE_ROOT}/systemd/"
  ssh_ok "mkdir -p $(printf '%q' "$REMOTE_ROOT/systemd")"
  rsync -az "$ROOT/systemd/" "${HOST}:${REMOTE_ROOT}/systemd/"
fi

echo "[*] restore +x on remote scripts"
remote_chmod_targets=""
for s in "${SCRIPTS[@]}"; do
  remote_chmod_targets+=" scripts/${s}"
done
ssh_ok "cd $(printf '%q' "$REMOTE_ROOT") && chmod +x${remote_chmod_targets} scripts/tinyfolder-*.sh 2>/dev/null; true"

after_ts="$(ssh_ok 'systemctl show gzmo-daemon -p ActiveEnterTimestamp --value' 2>/dev/null || echo unknown)"
daemon_state="$(ssh_ok 'systemctl is-active gzmo-daemon' 2>/dev/null || echo unknown)"

if [[ "$before_ts" != "unknown" && "$after_ts" != "unknown" && "$before_ts" != "$after_ts" ]]; then
  echo "[!] WARN: gzmo-daemon ActiveEnterTimestamp changed during sync ($before_ts → $after_ts)" >&2
  echo "    (script did not restart; something else may have.)" >&2
else
  echo "[OK] dual-writer safe — daemon ActiveEnterTimestamp unchanged ($before_ts)"
fi

# Spot-check execute bits
ssh_ok "test -x ${REMOTE_ROOT}/scripts/brain-feed-check.sh && test -x ${REMOTE_ROOT}/scripts/keep-quality-gate.sh" \
  || { echo "[!] remote +x check failed" >&2; exit 1; }

echo "[OK] brain-feed script/doc sync → ${HOST}:${REMOTE_ROOT}"
echo "    daemon=$daemon_state  (no restart by this script)"
echo ""
echo "Verify:"
echo "  ssh $HOST 'ls -l ${REMOTE_ROOT}/scripts/brain-feed-check.sh'"
echo "  bash scripts/brain-feed-check.sh"
echo "  bash scripts/opportunity-discovery-check.sh"
