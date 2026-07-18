#!/usr/bin/env bash
# Thin local status JSON for AOS Intelligence Dashboard (file poll / static HTTP).
# Shape matches AOS TelemetryPayload fields where possible.
#
#   bash scripts/aos-status-feed.sh
#   bash scripts/aos-status-feed.sh --serve   # python http.server :8765 (Ctrl-C to stop)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/aos-status"
SERVE=0
for a in "$@"; do
  case "$a" in
    --serve) SERVE=1 ;;
  esac
done
mkdir -p "$OUT"
export DATA OUT ROOT

python3 - <<'PY'
import json, os
from datetime import datetime, timezone
from pathlib import Path

data = Path(os.environ["DATA"])
out = Path(os.environ["OUT"])


def load(p: Path):
    try:
        return json.loads(p.read_text(encoding="utf-8"))
    except Exception:
        return None


arena = load(data / "arena" / "latest.json") or {}
euro = load(data / "arena" / "euro-night.json") or {}
wd = load(data / "scheduler-runs" / "latest-watchdog.json") or {}
faith = load(data / "faithfulness" / "latest.json") or {}
gate = load(data / "concept-gate" / "latest.json") or {}
hsp = load(data / "hsp-metabolism" / "latest.json") or {}
board = load(data / "arena" / "scoreboard.json") or {}

stale = bool(wd.get("stale"))
status = "error" if stale else "online"

# AOS TelemetryPayload-compatible core + GZMO extensions under `gzmo`.
payload = {
    "status": status,
    "current_model": arena.get("champion") or arena.get("engine"),
    "active_host": "workstation-nightburst",
    "backend_reachable": not stale,
    "energy_avg": arena.get("watts_avg") or arena.get("joules"),
    "z_score": arena.get("z"),
    "quality": arena.get("quality"),
    "price_ct_kwh": arena.get("electricity_c_kwh"),
    "message": (
        f"€/night={euro.get('euro_night_total')} · "
        f"watchdog={'STALE' if stale else 'fresh'} · "
        f"faith={faith.get('supported')}/{faith.get('total')} · "
        f"gate={gate.get('verdict')}"
    ),
    "gzmo": {
        "schema": "gzmo.aos.status/v1",
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "euro_night_total": euro.get("euro_night_total"),
        "arena_euro_cost": arena.get("euro_cost"),
        "watchdog_stale": stale,
        "concept_gate": gate.get("verdict"),
        "faithfulness_ok": faith.get("ok"),
        "hsp_events": len(hsp.get("events") or []),
        "scoreboard_html": str(data / "arena" / "scoreboard.html"),
        "scoreboard_generated": board.get("generated_at"),
    },
}

(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
# Convenience alias for simple GET /telemetry.json when --serve
(out / "telemetry.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"path": str(out / "latest.json"), "status": status, "z_score": payload.get("z_score"), "euro_night_total": euro.get("euro_night_total")}, indent=2))
PY

if [[ "$SERVE" -eq 1 ]]; then
  echo "[*] Serving $OUT on http://127.0.0.1:8765/telemetry.json (Ctrl-C to stop)"
  cd "$OUT"
  exec python3 -m http.server 8765 --bind 127.0.0.1
fi
