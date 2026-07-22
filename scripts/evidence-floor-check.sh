#!/usr/bin/env bash
# O14 — evidence-locate fixture floor (local; cognition already attaches spans).
#   bash scripts/evidence-floor-check.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CLONE="${GZMO_CLONE_ROOT:-$(dirname "$ROOT")}"
EV="$CLONE/evidence-locate"
OUT="${GZMO_DATA_NEXT:-$ROOT/data-next}/evidence-floor"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$CLONE/temp-bench/target}"
mkdir -p "$OUT"
[[ -d "$EV" ]] || { echo "missing $EV" >&2; exit 2; }
if [[ ! -x "$CARGO_TARGET_DIR/release/evidence-locate" ]]; then
  (cd "$EV" && cargo build --release -q)
fi
FIXTURE="$EV/fixtures/cases.json"
REPORT="$OUT/batch.json"
"$CARGO_TARGET_DIR/release/evidence-locate" batch --fixture "$FIXTURE" -o "$REPORT"
python3 - "$REPORT" "$OUT" <<'PY'
import json, sys
from datetime import datetime, timezone
from pathlib import Path
rep, out = Path(sys.argv[1]), Path(sys.argv[2])
d = json.loads(rep.read_text())
passed = int(d.get("passed") or 0)
total = int(d.get("total") or 0)
ok = total > 0 and passed == total
payload = {
    "schema": "gzmo.evidence_floor.check/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "ok": ok,
    "passed": passed,
    "total": total,
    "advice": "evidence_floor_ok" if ok else "evidence_floor_fail",
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n")
print(json.dumps(payload, indent=2))
raise SystemExit(0 if ok else 1)
PY
