#!/usr/bin/env bash
# Unpark Wave 3.1 demable: Arena lab observability snapshot (never starts gzmo-daemon jobs).
#   bash scripts/arena-lab-demo.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/arena-lab"
ARENA="${OBOLUS_ARENA_ROOT:-$HOME/github-clone/obolus-arena}"
mkdir -p "$OUT"

# Soft €/night stub from RAPL log if present
POWER=""
for p in "$ROOT/data/power.jsonl" "$DATA/power.jsonl"; do
  [[ -f "$p" ]] && POWER="$p" && break
done

export OUT ARENA POWER ROOT
python3 - <<'PY'
import json, os
from datetime import datetime, timezone
from pathlib import Path

out = Path(os.environ["OUT"])
arena = Path(os.environ.get("ARENA", ""))
power = Path(os.environ["POWER"]) if os.environ.get("POWER") else None
lines = 0
last = None
if power and power.is_file():
    raw = power.read_text(encoding="utf-8", errors="replace").splitlines()
    lines = len(raw)
    if raw:
        try:
            last = json.loads(raw[-1])
        except Exception:
            last = {"raw": raw[-1][:200]}

payload = {
    "schema": "gzmo.unpark.arena_lab.demo/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "wave": "3.1",
    "ok": True,
    "arena_sibling": str(arena) if arena.is_dir() else None,
    "rapl": {"path": str(power) if power else None, "lines": lines, "last": last},
    "euro_night_stub": {
        "note": "Aggregate Awattar × RAPL in sibling Arena; GZMO only observes",
        "estimated": False,
    },
    "daemon_jobs_touched": False,
    "advice": "arena_lab_demo_ok — observability snapshot; daemon untouched",
}
(out / "demo.json").write_text(json.dumps(payload, indent=2) + "\n")
(out / "lab-night.md").write_text(
    "# Arena lab night (Unpark Wave 3)\n\n"
    f"- Sibling: `{payload['arena_sibling'] or 'not found'}`\n"
    f"- RAPL lines: {lines}\n"
    "- Do **not** add Arena cron to gzmo-daemon.\n"
    "- Run overnight z-loops from sibling `obolus-arena/`.\n",
    encoding="utf-8",
)
print(json.dumps({"ok": True, "advice": payload["advice"], "arena": payload["arena_sibling"]}, indent=2))
PY

# Prefer sibling path for check
OBOLUS_ARENA_ROOT="$ARENA" bash "$ROOT/scripts/arena-lab-check.sh"
echo "[OK] Arena lab demo → $OUT"
