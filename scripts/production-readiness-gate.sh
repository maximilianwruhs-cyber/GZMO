#!/usr/bin/env bash
# Co-primary production readiness: Goal A (product MCP) + Goal C (living CT101).
# Exit 0 when both gates GREEN (HOLD rows allowed). Writes combined artifact.
#
#   bash scripts/production-readiness-gate.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/production-readiness"
mkdir -p "$OUT"
LOG="$OUT/gate.log"
: >"$LOG"

echo "=== Production readiness (A + C) ===" | tee -a "$LOG"

set +e
bash "$ROOT/scripts/product-readiness-gate.sh" >>"$LOG" 2>&1
prod_rc=$?
bash "$ROOT/scripts/living-readiness-gate.sh" >>"$LOG" 2>&1
live_rc=$?
set -e

export OUT DATA prod_rc live_rc
python3 - <<'PY'
import json, os
from datetime import datetime, timezone
from pathlib import Path

out = Path(os.environ["OUT"])
data = Path(os.environ["DATA"])
prod_path = data / "product-readiness" / "latest.json"
live_path = data / "living-readiness" / "latest.json"
prod = json.loads(prod_path.read_text()) if prod_path.is_file() else {"verdict": "RED", "advice": "missing"}
live = json.loads(live_path.read_text()) if live_path.is_file() else {"verdict": "RED", "advice": "missing"}
prod_ok = prod.get("verdict") == "GREEN" and int(os.environ["prod_rc"]) == 0
live_ok = live.get("verdict") == "GREEN" and int(os.environ["live_rc"]) == 0
verdict = "GREEN" if prod_ok and live_ok else "RED"
advice = (
    "production_ready — product (A) + living (C) gates GREEN"
    if verdict == "GREEN"
    else "production_not_ready — see product-readiness/ and living-readiness/"
)
payload = {
    "schema": "gzmo.production.readiness/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "verdict": verdict,
    "ok": verdict == "GREEN",
    "advice": advice,
    "goals": {
        "A": {
            "name": "product_mcp",
            "verdict": prod.get("verdict"),
            "advice": prod.get("advice"),
            "counts": prod.get("counts"),
            "exit": int(os.environ["prod_rc"]),
        },
        "C": {
            "name": "living_appliance",
            "verdict": live.get("verdict"),
            "advice": live.get("advice"),
            "counts": live.get("counts"),
            "exit": int(os.environ["live_rc"]),
            "intentional_holds": [
                "living-appliance-smoke (workstation throwaway Neo4j / no pin .env)",
                "living-appliance-pin-ct101 (pre-promote drift vs /opt/database-cluster)",
            ],
        },
    },
    "artifacts": {
        "product": str(prod_path),
        "living": str(live_path),
    },
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
md = [
    "# Production readiness (A + C)",
    "",
    f"Verdict: **{verdict}**",
    "",
    f"- **A product:** {prod.get('verdict')} — {prod.get('advice')}",
    f"- **C living:** {live.get('verdict')} — {live.get('advice')}",
    "",
    "See docs/PRODUCT_PRODUCTION_READINESS.md and docs/LIVING_PRODUCTION_READINESS.md",
]
(out / "latest.md").write_text("\n".join(md) + "\n", encoding="utf-8")
print(json.dumps({"verdict": verdict, "advice": advice, "A": prod.get("verdict"), "C": live.get("verdict")}, indent=2))
raise SystemExit(0 if verdict == "GREEN" else 1)
PY
GATE_EXIT=$?
echo "=== production readiness done (exit $GATE_EXIT) ===" | tee -a "$LOG"
exit "$GATE_EXIT"
