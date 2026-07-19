#!/usr/bin/env bash
# Unpark Wave 2.3 demable: emit a Synapse-shaped motif event file for HSP consumers,
# then sonify metabolism + emit into MIDI/WAV (no daemon, not on living GREEN gate).
#
#   bash scripts/hsp-emit-demo.sh
#   bash scripts/hsp-emit-demo.sh --play   # optional aplay/paplay / hsp ping
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/hsp-emit"
mkdir -p "$OUT/events"

SONIFY_EXTRA=()
for a in "$@"; do
  case "$a" in
    --play) SONIFY_EXTRA+=(--play) ;;
  esac
done

TS="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
EVENT="$OUT/events/motif-$(date -u +%Y%m%dT%H%M%SZ).json"
python3 - <<PY
import json
from pathlib import Path
ev = {
  "schema": "gzmo.hsp.motif/v1",
  "generated_at": "$TS",
  "source": "gzmo.unpark.hsp_emit_demo",
  "motif": "distill_tick",
  "intensity": 0.35,
  "notes": "Lab emit for HSP — does not gate living GREEN",
  "synapse_hint": "EventType::HealthTick / distill completion may map here",
  "sonify": "scripts/hsp-metabolism-sonify.sh → data-next/hsp-metabolism/",
}
Path("$EVENT").write_text(json.dumps(ev, indent=2) + "\n")
latest = Path("$OUT/latest-event.json")
latest.write_text(json.dumps(ev, indent=2) + "\n")
print(json.dumps({"ok": True, "event": "$EVENT"}, indent=2))
PY

# Sonify after emit so latest-event.json is visible to the motif mapper.
bash "$ROOT/scripts/hsp-metabolism-sonify.sh" "${SONIFY_EXTRA[@]}"

bash "$ROOT/scripts/hsp-emit-check.sh"
echo "[OK] HSP emit demo → $EVENT (+ sonify under $DATA/hsp-metabolism/)"
