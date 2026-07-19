#!/usr/bin/env bash
# Unpark Wave 3.3 demable: write recommend-only forge pin stub.
#   bash scripts/forge-lab-demo.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/forge-lab"
mkdir -p "$OUT"

python3 - <<PY
import json
from datetime import datetime, timezone
from pathlib import Path
out = Path("$OUT")
rec = {
  "schema": "gzmo.unpark.forge.recommend/v1",
  "generated_at": datetime.now(timezone.utc).isoformat(),
  "wave": "3.3",
  "action": "recommend",
  "blocks_distill": False,
  "pins": [
    {"organ": "example-winner", "reason": "stub — replace from sibling forge overnight"},
  ],
  "note": "Display/route advice only",
}
(out / "recommend.json").write_text(json.dumps(rec, indent=2) + "\n")
print(json.dumps(rec, indent=2))
PY
bash "$ROOT/scripts/forge-lab-check.sh"
echo "[OK] Forge recommend stub → $OUT/recommend.json"
