#!/usr/bin/env bash
# Knowledge-loop honesty: run honeypot-gate lifecycle goldens (ADR-0005 Ring 3/4).
#   bash scripts/lifecycle-goldens-check.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CLONE="${GZMO_CLONE_ROOT:-$(dirname "$ROOT")}"
GATE="${CLONE}/honeypot-gate"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$CLONE/temp-bench/target}"
GOLDENS="$GATE/fixtures/lifecycle-goldens.json"
LAB_GOLDENS="${LITTLE_TOOLS_LAB_ROOT:-$CLONE/little-tools-lab}/fixtures/honeypot-lifecycle/lifecycle-goldens.json"
OUT="${GZMO_DATA_NEXT:-$ROOT/data-next}/lifecycle-goldens"
mkdir -p "$OUT"

[[ -f "$GOLDENS" ]] || { echo "missing $GOLDENS" >&2; exit 2; }
if [[ -f "$LAB_GOLDENS" ]]; then
  if ! cmp -s "$GOLDENS" "$LAB_GOLDENS"; then
    echo "error: LTL lifecycle goldens drift — sync honeypot-gate ↔ little-tools-lab fixtures" >&2
    exit 1
  fi
  echo "=== LTL lifecycle goldens sync ok ==="
fi

run_one() {
  local label="$1"
  shift
  echo "=== $label ==="
  local out
  out="$("$@" 2>&1)" || {
    echo "$out" >&2
    exit 1
  }
  echo "$out"
  if ! echo "$out" | grep -qE 'test result: ok\. [1-9][0-9]* passed'; then
    echo "error: expected ≥1 passing test for $label" >&2
    exit 1
  fi
}

run_one "lifecycle goldens (honeypot-gate)" \
  bash -c "cd \"$GATE\" && cargo test --lib lifecycle::tests::lifecycle_goldens_fixture -- --exact"
run_one "lifecycle goldens (gzmo-core mirror)" \
  bash -c "cd \"$ROOT\" && cargo test -p gzmo-core memory::lifecycle::tests::lifecycle_goldens_mirror_honeypot_gate_fixture -- --exact"

CRAFT="${LITTLE_TOOLS_LAB_ROOT:-$CLONE/little-tools-lab}/scripts/craft-goldens-check.sh"
if [[ -f "$CRAFT" ]]; then
  echo "=== LTL craft-goldens-check ==="
  bash "$CRAFT"
fi

python3 - "$GOLDENS" "$OUT" <<'PY'
import json, sys
from datetime import datetime, timezone
from pathlib import Path
goldens = Path(sys.argv[1])
out = Path(sys.argv[2])
doc = json.loads(goldens.read_text(encoding="utf-8"))
payload = {
    "schema": "gzmo.lifecycle_goldens.check/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "ok": True,
    "advice": "lifecycle_goldens_ok",
    "cases": len(doc.get("cases") or []),
    "fixture": str(goldens),
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"ok": True, "cases": payload["cases"]}, indent=2))
PY
