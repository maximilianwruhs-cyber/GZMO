#!/usr/bin/env bash
# Unpark Wave 3.1 demable: Arena lab observability snapshot (never starts gzmo-daemon jobs).
# Chains RAPL probe + €/night aggregate when available.
#
#   bash scripts/arena-lab-demo.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/arena-lab"
ARENA="${OBOLUS_ARENA_ROOT:-$HOME/github-clone/obolus-arena}"
mkdir -p "$OUT"

# Soft probes — never fail the demo if ACL / sibling history is missing.
bash "$ROOT/scripts/rapl-probe.sh" >/tmp/arena-lab-rapl-probe.log 2>&1 || true
bash "$ROOT/scripts/euro-night-aggregate.sh" >/tmp/arena-lab-euro-night.log 2>&1 || true

# Soft €/night stub from RAPL log if present
POWER=""
for p in "$ROOT/data/power.jsonl" "$DATA/power.jsonl"; do
  [[ -f "$p" ]] && POWER="$p" && break
done

EURO_NIGHT=""
[[ -f "$DATA/arena/euro-night.json" ]] && EURO_NIGHT="$DATA/arena/euro-night.json"
RAPL_LATEST=""
[[ -f "$DATA/rapl/latest.json" ]] && RAPL_LATEST="$DATA/rapl/latest.json"

export OUT ARENA POWER ROOT EURO_NIGHT RAPL_LATEST
python3 - <<'PY'
import json, os
from datetime import datetime, timezone
from pathlib import Path

out = Path(os.environ["OUT"])
arena = Path(os.environ.get("ARENA", ""))
power = Path(os.environ["POWER"]) if os.environ.get("POWER") else None
euro_path = Path(os.environ["EURO_NIGHT"]) if os.environ.get("EURO_NIGHT") else None
rapl_path = Path(os.environ["RAPL_LATEST"]) if os.environ.get("RAPL_LATEST") else None

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

euro = None
if euro_path and euro_path.is_file():
    try:
        euro = json.loads(euro_path.read_text(encoding="utf-8"))
    except Exception:
        euro = {"error": "unreadable euro-night.json"}

rapl = None
if rapl_path and rapl_path.is_file():
    try:
        rapl = json.loads(rapl_path.read_text(encoding="utf-8"))
    except Exception:
        rapl = {"error": "unreadable rapl latest.json"}

payload = {
    "schema": "gzmo.unpark.arena_lab.demo/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "wave": "3.1",
    "ok": True,
    "arena_sibling": str(arena) if arena.is_dir() else None,
    "rapl": {"path": str(power) if power else None, "lines": lines, "last": last},
    "rapl_probe": {
        "path": str(rapl_path) if rapl_path else None,
        "readable": bool(rapl and (rapl.get("readable_paths") or [])),
        "summary": {
            k: rapl.get(k)
            for k in ("ok", "readable_paths", "sample_delta_j_0_2s", "advice", "note")
            if rapl and k in rapl
        }
        if rapl
        else None,
    },
    "euro_night": {
        "path": str(euro_path) if euro_path else None,
        "euro_night_total": (euro or {}).get("euro_night_total"),
        "arena_euro_sum": (euro or {}).get("arena_euro_sum"),
        "metabolism_euro_est": (euro or {}).get("metabolism_euro_est"),
        "note": "Aggregate Awattar × RAPL in sibling Arena; GZMO only observes",
        "estimated": bool((euro or {}).get("metabolism_euro_est") is not None),
    },
    "daemon_jobs_touched": False,
    "advice": "arena_lab_demo_ok — RAPL probe + €/night chained; daemon untouched",
}
(out / "demo.json").write_text(json.dumps(payload, indent=2) + "\n")
euro_line = payload["euro_night"]["euro_night_total"]
(out / "lab-night.md").write_text(
    "# Arena lab night (Unpark Wave 3)\n\n"
    f"- Sibling: `{payload['arena_sibling'] or 'not found'}`\n"
    f"- RAPL power.jsonl lines: {lines}\n"
    f"- RAPL probe: `{payload['rapl_probe']['path'] or 'none'}` "
    f"(readable={payload['rapl_probe']['readable']})\n"
    f"- €/night total: `{euro_line}`\n"
    "- Do **not** add Arena cron to gzmo-daemon.\n"
    "- Run overnight z-loops from sibling `obolus-arena/`.\n",
    encoding="utf-8",
)
print(
    json.dumps(
        {
            "ok": True,
            "advice": payload["advice"],
            "arena": payload["arena_sibling"],
            "euro_night_total": euro_line,
            "rapl_readable": payload["rapl_probe"]["readable"],
        },
        indent=2,
    )
)
PY

# Prefer sibling path for check
OBOLUS_ARENA_ROOT="$ARENA" bash "$ROOT/scripts/arena-lab-check.sh"
echo "[OK] Arena lab demo → $OUT"
