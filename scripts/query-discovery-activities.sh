#!/usr/bin/env bash
# Query remediation tracker + spawn snapshots (Jules JQL analogue, jq-based).
# Usage:
#   query-discovery-activities.sh failed          # findings with failed verify
#   query-discovery-activities.sh open            # open/in_flight findings
#   query-discovery-activities.sh snapshots       # list snapshot files
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SKILLS="${GZMO_SKILLS_ROOT:-$HOME/gzmo_skills}"
TRACKER="${PI_MENTOR_DISCOVERY_TRACKER:-$SKILLS/data/pi-mentor-discovery/remediation-tracker.json}"
SNAPSHOTS="${GZMO_DISCOVERY_SNAPSHOTS:-$SKILLS/data/discovery-implementation/snapshots}"

MODE="${1:-summary}"

case "$MODE" in
  failed)
    jq '[.findings[] | select(.status == "failed")]' "$TRACKER" 2>/dev/null || echo '[]'
    ;;
  open)
    jq '[.findings[] | select(.status == "open" or .status == "in_flight" or .status == "probed")]' "$TRACKER" 2>/dev/null || echo '[]'
    ;;
  snapshots)
    if [[ -d "$SNAPSHOTS" ]]; then
      find "$SNAPSHOTS" -maxdepth 1 -name '*.json' -printf '%f\n' | sort
    else
      echo "no snapshots dir: $SNAPSHOTS"
    fi
    ;;
  summary|*)
    jq '{
      open: [.findings[] | select(.status == "open")] | length,
      in_flight: [.findings[] | select(.status == "in_flight")] | length,
      probed: [.findings[] | select(.status == "probed")] | length,
      fixed: [.findings[] | select(.status == "fixed")] | length,
      failed: [.findings[] | select(.status == "failed")] | length,
      total: (.findings | length)
    }' "$TRACKER" 2>/dev/null || echo '{"error":"tracker not found"}'
    ;;
esac
