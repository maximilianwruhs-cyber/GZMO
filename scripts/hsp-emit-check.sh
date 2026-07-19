#!/usr/bin/env bash
# Unpark Wave 2.3: HSP / Synapse emit hooks readiness (not on GREEN overnight gate).
#   bash scripts/hsp-emit-check.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/hsp-emit"
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
EOF
row PASS "emit-contract" "$OUT/emit-contract.md"

# Soft: sibling HSP path if present
HSP_ROOT="${HSP_ROOT:-$HOME/github-clone/HSP}"
if [[ -d "$HSP_ROOT" ]]; then
  row PASS "hsp-sibling" "$HSP_ROOT"
else
  row HOLD "hsp-sibling" "HSP repo not at HSP_ROOT — contract only"
fi

ROWS_TSV="$(printf '%s\n' "${ROWS[@]}")"
export OUT pass fail hold ROWS_TSV
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
advice="hsp_emit_ok" if fail_n==0 and hold_n==0 else ("hsp_emit_hold" if fail_n==0 else "hsp_emit_fail")
payload={"schema":"gzmo.unpark.hsp_emit/v1","generated_at":datetime.now(timezone.utc).isoformat(),
  "verdict":verdict,"ok":fail_n==0,"advice":advice,
  "counts":{"pass":pass_n,"fail":fail_n,"hold":hold_n},"wave":"2.3","checks":checks,
  "note":"Not on living GREEN overnight gate."}
(out/"latest.json").write_text(json.dumps(payload,indent=2)+"\n")
print(json.dumps({"verdict":verdict,"advice":advice,"pass":pass_n,"fail":fail_n,"hold":hold_n},indent=2))
raise SystemExit(0 if fail_n==0 else 1)
PY
