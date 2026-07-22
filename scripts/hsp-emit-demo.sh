#!/usr/bin/env bash
# Unpark Wave 2.3 demable: emit a Synapse-shaped motif event for HSP consumers,
# then sonify metabolism + emit into MIDI/WAV (no daemon, not on living GREEN gate).
#
#   bash scripts/hsp-emit-demo.sh
#   bash scripts/hsp-emit-demo.sh --play
#   bash scripts/hsp-emit-demo.sh --motif spark_flare --intensity 0.8 --play
#
# Theater only — not Brain Feed, not living GREEN.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/hsp-emit"
mkdir -p "$OUT/events"

MOTIF="distill_tick"
INTENSITY="0.35"
SONIFY_EXTRA=()
args=("$@")
i=0
while (( i < ${#args[@]} )); do
  case "${args[$i]}" in
    --play) SONIFY_EXTRA+=(--play) ;;
    --motif)
      i=$((i + 1))
      MOTIF="${args[$i]:-distill_tick}"
      ;;
    --intensity)
      i=$((i + 1))
      INTENSITY="${args[$i]:-0.35}"
      ;;
  esac
  i=$((i + 1))
done

TS="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
EVENT="$OUT/events/motif-$(date -u +%Y%m%dT%H%M%SZ).json"
export EVENT TS MOTIF INTENSITY OUT
python3 - <<'PY'
import json, os
from pathlib import Path
ev = {
  "schema": "gzmo.hsp.motif/v1",
  "generated_at": os.environ["TS"],
  "source": "gzmo.unpark.hsp_emit_demo",
  "motif": os.environ["MOTIF"],
  "intensity": float(os.environ["INTENSITY"]),
  "notes": "Lab emit for HSP — does not gate living GREEN",
  "synapse_hint": "EventType::HealthTick / distill completion may map here",
  "sonify": "scripts/hsp-metabolism-sonify.sh → data-next/hsp-metabolism/",
  "theater": True,
  "brain_feed": False,
}
Path(os.environ["EVENT"]).write_text(json.dumps(ev, indent=2) + "\n")
Path(os.environ["OUT"], "latest-event.json").write_text(json.dumps(ev, indent=2) + "\n")
print(json.dumps({"ok": True, "event": os.environ["EVENT"], "motif": ev["motif"]}, indent=2))
PY

bash "$ROOT/scripts/hsp-metabolism-sonify.sh" "${SONIFY_EXTRA[@]}"
bash "$ROOT/scripts/hsp-emit-check.sh"
echo "[OK] HSP emit demo → $EVENT (+ sonify under $DATA/hsp-metabolism/)"
echo "     Theater only — not Brain Feed / not living GREEN."
