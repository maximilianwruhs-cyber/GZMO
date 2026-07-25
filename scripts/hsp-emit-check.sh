#!/usr/bin/env bash
# Unpark Wave 2.3: HSP / Synapse emit + sonify readiness (not on GREEN overnight gate).
#   bash scripts/hsp-emit-check.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/hsp-emit"
SONIFY_OUT="$DATA/hsp-metabolism"
mkdir -p "$OUT"
pass=0; fail=0; hold=0
declare -a ROWS=()
row() { local s="$1" n="$2" d="$3"; ROWS+=("$s|$n|$d"); case "$s" in PASS) pass=$((pass+1));; FAIL) fail=$((fail+1));; HOLD) hold=$((hold+1));; esac; echo "[$s] $n — $d"; }

echo "=== HSP emit check (Unpark W2.3) ==="
row PASS "scope" "sonification hooks — not living GREEN gate"

# Synapse ownership / event types exist in repo
if rg -n 'SynapseEvent|EventType::' "$ROOT/gzmo-core/src" --glob '*.rs' 2>/dev/null | head -1 >/dev/null; then
  row PASS "synapse-types" "SynapseEvent types in gzmo-core"
else
  row HOLD "synapse-types" "no SynapseEvent hits"
fi

# Document emit contract for sibling HSP
cat >"$OUT/emit-contract.md" <<'EOF'
# HSP emit contract (Unpark Wave 2.3)

GZMO may append Synapse events for distill/dream/embed. Sibling HSP consumes motifs.

- Not required for `living-readiness-gate` GREEN.
- Do not block overnight metabolism on MIDI/WAV availability.
- Preferred bus: Synapse on living host; lab may use file drops under `data-next/hsp-emit/`.
- Sonify front door: `bash scripts/hsp-metabolism-sonify.sh` (also chained from `hsp-emit-demo.sh`)
  writes `data-next/hsp-metabolism/{latest.mid,latest.wav,latest.json}` from metabolism
  artifacts + the latest `hsp-emit` motif. Optional `--play` preflights sink
  volume, then pw-play/paplay/aplay / `hsp ping`.
EOF
row PASS "emit-contract" "$OUT/emit-contract.md"

[[ -x "$ROOT/scripts/hsp-metabolism-sonify.sh" ]] \
  && row PASS "sonify-script" "hsp-metabolism-sonify.sh executable" \
  || row FAIL "sonify-script" "missing or not executable"

if [[ -f "$OUT/latest-event.json" ]]; then
  if python3 -c "
import json
d=json.load(open('$OUT/latest-event.json'))
notes=(d.get('notes') or '').lower()
ok=(
  d.get('schema')=='gzmo.hsp.motif/v1'
  and bool(d.get('motif'))
  and ('lab' in notes or ('not' in notes and 'green' in notes))
)
raise SystemExit(0 if ok else 1)
"; then
    row PASS "emit-event" "motif schema + lab/not-GREEN notes"
  else
    row FAIL "emit-event" "latest-event.json incomplete — rerun hsp-emit-demo.sh"
  fi
else
  row HOLD "emit-event" "run hsp-emit-demo.sh to drop latest-event.json"
fi

if [[ -f "$SONIFY_OUT/latest.mid" && -f "$SONIFY_OUT/latest.wav" ]]; then
  if python3 -c "
from pathlib import Path
mid=Path('$SONIFY_OUT/latest.mid'); wav=Path('$SONIFY_OUT/latest.wav')
raise SystemExit(0 if mid.is_file() and wav.is_file() and mid.stat().st_size>0 and wav.stat().st_size>0 else 1)
"; then
    row PASS "sonify-artifacts" "$SONIFY_OUT/latest.{mid,wav} non-empty"
  else
    row FAIL "sonify-artifacts" "MIDI/WAV empty — rerun hsp-emit-demo.sh"
  fi
else
  row HOLD "sonify-artifacts" "no MIDI/WAV yet — run hsp-emit-demo.sh or hsp-metabolism-sonify.sh"
fi

# Soft: sibling HSP path if present
HSP_ROOT="${HSP_ROOT:-$HOME/github-clone/HSP}"
if [[ -d "$HSP_ROOT" ]]; then
  row PASS "hsp-sibling" "$HSP_ROOT"
else
  row HOLD "hsp-sibling" "HSP repo not at HSP_ROOT — contract only"
fi

# Soft: playback path readiness (agents often misblame WAV format when sink is muted/low)
if command -v wpctl >/dev/null 2>&1 && wpctl status >/dev/null 2>&1; then
  vol="$(wpctl get-volume @DEFAULT_AUDIO_SINK@ 2>/dev/null | awk '{print $2}')"
  sink="$(wpctl status 2>/dev/null | awk '/Sinks:/{s=1;next} /Sources:/{s=0} s && /\*/ {gsub(/^[[:space:]]+/,""); print; exit}')"
  if awk -v v="${vol:-0}" 'BEGIN{exit !(v+0 < 0.15)}'; then
    row HOLD "audio-play" "PipeWire up but default sink vol=${vol:-?} (<0.15) — --play may be inaudible; wpctl set-volume @DEFAULT_AUDIO_SINK@ 0.4"
  else
    row PASS "audio-play" "PipeWire default sink ready (vol=${vol}; ${sink:-sink})"
  fi
elif command -v aplay >/dev/null 2>&1 || command -v pw-play >/dev/null 2>&1; then
  row HOLD "audio-play" "player present but wpctl missing — cannot verify sink volume"
else
  row HOLD "audio-play" "no aplay/pw-play — generate-only; --play will skip"
fi

ROWS_TSV="$(printf '%s\n' "${ROWS[@]}")"
export OUT pass fail hold ROWS_TSV SONIFY_OUT
python3 - <<'PY'
import json, os
from datetime import datetime, timezone
from pathlib import Path
out=Path(os.environ["OUT"]); checks={}
for line in os.environ.get("ROWS_TSV","").splitlines():
    if not line.strip(): continue
    st,n,d=line.split("|",2); checks[n]={"status":st,"detail":d}
fail_n=int(os.environ["fail"]); hold_n=int(os.environ["hold"]); pass_n=int(os.environ["pass"])
verdict="GREEN" if fail_n==0 else "RED"
emit_ok = checks.get("emit-event",{}).get("status")=="PASS"
sonify_ok = checks.get("sonify-artifacts",{}).get("status")=="PASS"
if fail_n==0 and emit_ok and sonify_ok:
    advice="hsp_emit_ok — motif emit + MIDI/WAV (not living GREEN gate)"
elif fail_n==0:
    advice="hsp_emit_hold — run hsp-emit-demo.sh for emit+sonify evidence"
else:
    advice="hsp_emit_fail"
payload={"schema":"gzmo.unpark.hsp_emit/v1","generated_at":datetime.now(timezone.utc).isoformat(),
  "verdict":verdict,"ok":fail_n==0,"advice":advice,
  "counts":{"pass":pass_n,"fail":fail_n,"hold":hold_n},"wave":"2.3","checks":checks,
  "sonify_dir":os.environ.get("SONIFY_OUT"),
  "emit_ok": emit_ok, "sonify_ok": sonify_ok,
  "note":"Not on living GREEN overnight gate."}
(out/"latest.json").write_text(json.dumps(payload,indent=2)+"\n")
print(json.dumps({"verdict":verdict,"advice":advice,"pass":pass_n,"fail":fail_n,"hold":hold_n},indent=2))
raise SystemExit(0 if fail_n==0 else 1)
PY
