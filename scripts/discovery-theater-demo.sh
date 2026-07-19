#!/usr/bin/env bash
# Unpark Wave 2.2 demable: session-prep checklist (theater ≠ living scout KPI).
#   bash scripts/discovery-theater-demo.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/discovery-theater"
mkdir -p "$OUT"

cat >"$OUT/session-prep.md" <<'EOF'
# Mutual-discovery theater — session prep (Unpark Wave 2.2)

Theater is human pedagogy. Do **not** redefine living publish/timer KPI.

1. Open [MUTUAL_DISCOVERY_THEATER.md](../../docs/MUTUAL_DISCOVERY_THEATER.md)
2. Keep Forum-1 / scout path under [DISCOVERY_LIFECYCLE.md](../../docs/DISCOVERY_LIFECYCLE.md)
3. Confirm `living-readiness-gate` has no theater rows
4. Optional: walk one Socratic LINK pack from `docs/research/mutual-discovery/`
EOF

bash "$ROOT/scripts/discovery-theater-check.sh"
python3 - <<PY
import json
from datetime import datetime, timezone
from pathlib import Path
out = Path("$OUT")
payload = {
  "schema": "gzmo.unpark.discovery_theater.demo/v1",
  "generated_at": datetime.now(timezone.utc).isoformat(),
  "ok": True,
  "session_prep": str(out / "session-prep.md"),
  "wave": "2.2",
}
(out / "demo.json").write_text(json.dumps(payload, indent=2) + "\n")
print(json.dumps(payload, indent=2))
PY
echo "[OK] discovery theater demo → $OUT"
