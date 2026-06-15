#!/usr/bin/env bash
# Pi ↔ GZMO integration smoke (mentor bridge + distill parser).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"

echo "== Validating Pedagogy Graphs =="
"$ROOT/target/release/gzmo" pedagogy graph validate "$ROOT/data/pedagogy/graphs/"

echo "== Repo .mcp.json =="
"$ROOT/scripts/verify-mcp-json.sh"

"$ROOT/scripts/pi/test_mentor_dialog.sh"
"$ROOT/scripts/pi/test_distill_pi.sh"
"$ROOT/scripts/pi/test_session_end_distill.sh"
"$ROOT/scripts/pi/test_topic_shift_distill.sh"
"$ROOT/scripts/pi/test_synapse_session_correlation.sh"
"$ROOT/scripts/pi/test_forum_romanum_schema.sh"
"$ROOT/scripts/pi/test_forum_romanum_emit.sh"
"$ROOT/scripts/pi/test_kurator_monitor.sh"
"$ROOT/scripts/pi/test_synapse_skill_invoke.sh"
"$ROOT/scripts/pi/validate_routing.sh"
if [[ -S "$ROOT/data/gzmo_mentor.sock" ]]; then
  python3 "$ROOT/scripts/pi/test_mcp_mentor.py"
else
  echo "SKIP MCP mentor test (daemon socket missing — start gzmo-daemon first)" >&2
fi
echo "OK: Pi integration smoke complete"
