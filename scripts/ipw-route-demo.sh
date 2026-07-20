#!/usr/bin/env bash
# Unpark Wave 3.2 demable: IpW advice matrix for chat vs heavy_bench.
# Advice only — never auto-block distill.
#
#   bash scripts/ipw-route-demo.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/ipw-route"
ROUTER="$DATA/ipw-router"
mkdir -p "$OUT"

run_task() {
  local task="$1" dest="$2"
  bash "$ROOT/scripts/ipw-route.sh" --task "$task" | tee "$OUT/advice-${task}.txt"
  if [[ -f "$ROUTER/latest.json" ]]; then
    cp "$ROUTER/latest.json" "$dest"
  else
    echo "{\"error\":\"missing ipw-router latest after $task\"}" >"$dest"
  fi
}

run_task chat "$OUT/route-chat.json"
run_task heavy_bench "$OUT/route-heavy.json"

export OUT
python3 - <<'PY'
import json
from datetime import datetime, timezone
from pathlib import Path

out = Path(__import__("os").environ["OUT"])
chat = json.loads((out / "route-chat.json").read_text(encoding="utf-8"))
heavy = json.loads((out / "route-heavy.json").read_text(encoding="utf-8"))
routes_diverge = chat.get("route") != heavy.get("route")
payload = {
    "schema": "gzmo.unpark.ipw.demo/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "ok": True,
    "wave": "3.2",
    "blocks_distill": False,
    "matrix": {
        "chat": {"task_class": chat.get("task_class"), "route": chat.get("route")},
        "heavy_bench": {
            "task_class": heavy.get("task_class"),
            "route": heavy.get("route"),
        },
        "routes_diverge": routes_diverge,
    },
    "artifacts": [
        "advice-chat.txt",
        "advice-heavy_bench.txt",
        "route-chat.json",
        "route-heavy.json",
    ],
    "note": "Advice only — never auto-block distill",
}
(out / "demo.json").write_text(json.dumps(payload, indent=2) + "\n")
(out / "matrix.md").write_text(
    "\n".join(
        [
            "# IpW route matrix",
            "",
            f"chat → **{chat.get('route')}**",
            f"heavy_bench → **{heavy.get('route')}**",
            f"diverge: `{routes_diverge}`",
            f"blocks_distill: `{payload['blocks_distill']}`",
            "",
        ]
    )
    + "\n",
    encoding="utf-8",
)
print(json.dumps(payload, indent=2))
if not routes_diverge:
    raise SystemExit("chat and heavy_bench routes did not diverge")
PY

bash "$ROOT/scripts/ipw-route-check.sh"
echo "[OK] IpW demo → $OUT"
