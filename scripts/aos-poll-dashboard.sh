#!/usr/bin/env bash
# Unpark Wave 1.4 demable: write a read-only intelligence dashboard JSON for AOS/sidebar.
#   bash scripts/aos-poll-dashboard.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/aos-poll"
HOST="${CT101_SSH_HOST:-ct101}"
mkdir -p "$OUT"

bash "$ROOT/scripts/aos-poll-check.sh" >/dev/null 2>&1 || true

export OUT DATA HOST ROOT
python3 - <<'PY'
import json, os, subprocess
from datetime import datetime, timezone
from pathlib import Path

out = Path(os.environ["OUT"])
data = Path(os.environ["DATA"])
host = os.environ["HOST"]

def load(p):
    try:
        return json.loads(Path(p).read_text())
    except Exception:
        return None

living = load(data / "living-readiness" / "latest.json") or {}
product = load(data / "product-readiness" / "latest.json") or {}
poll = load(out / "latest.json") or {}

daemon = "unknown"
sidecars = 0
try:
    r = subprocess.run(
        ["ssh", "-o", "ConnectTimeout=8", "-o", "BatchMode=yes", host,
         "systemctl is-active gzmo-daemon; docker ps --format '{{.Names}}' | grep -c sidecar || true"],
        capture_output=True, text=True, timeout=15,
    )
    lines = [x.strip() for x in r.stdout.splitlines() if x.strip()]
    if lines:
        daemon = lines[0]
    if len(lines) > 1 and lines[1].isdigit():
        sidecars = int(lines[1])
except Exception:
    pass

dash = {
    "schema": "gzmo.unpark.aos_dashboard/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "wave": "1.4",
    "arena_required": False,
    "living": {
        "verdict": living.get("verdict"),
        "advice": living.get("advice"),
        "daemon": daemon,
        "sidecars": sidecars,
    },
    "product": {
        "verdict": product.get("verdict"),
        "advice": product.get("advice"),
    },
    "poll": poll.get("advice"),
    "ok": True,
}
(out / "dashboard.json").write_text(json.dumps(dash, indent=2) + "\n")
(out / "dashboard.md").write_text(
    f"# AOS poll dashboard\n\n"
    f"- Living: **{dash['living']['verdict']}** ({daemon}, {sidecars} sidecars)\n"
    f"- Product: **{dash['product']['verdict']}**\n"
    f"- Arena required: no\n",
    encoding="utf-8",
)
print(json.dumps({"ok": True, "dashboard": str(out / "dashboard.json")}, indent=2))
PY
echo "[OK] AOS dashboard → $OUT/dashboard.json"
