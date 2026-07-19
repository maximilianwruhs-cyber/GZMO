#!/usr/bin/env bash
# Unpark Wave 3.2 demable: run IpW advice for chat + heavy_bench tasks.
#   bash scripts/ipw-route-demo.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/ipw-route"
mkdir -p "$OUT"

bash "$ROOT/scripts/ipw-route.sh" --task chat | tee "$OUT/advice-chat.txt"
bash "$ROOT/scripts/ipw-route.sh" --task heavy_bench | tee "$OUT/advice-heavy.txt" || true
bash "$ROOT/scripts/ipw-route-check.sh"
python3 - <<PY
import json
from datetime import datetime, timezone
from pathlib import Path
out = Path("$OUT")
payload = {
  "schema": "gzmo.unpark.ipw.demo/v1",
  "generated_at": datetime.now(timezone.utc).isoformat(),
  "ok": True,
  "wave": "3.2",
  "artifacts": ["advice-chat.txt", "advice-heavy.txt"],
  "note": "Advice only — never auto-block distill",
}
(out / "demo.json").write_text(json.dumps(payload, indent=2) + "\n")
print(json.dumps(payload, indent=2))
PY
echo "[OK] IpW demo → $OUT"
