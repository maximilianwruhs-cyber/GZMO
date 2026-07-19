#!/usr/bin/env bash
# Refresh GZMO → AOS status feed and verify file (and optional :8765) poll path.
#
#   bash scripts/aos-gzmo-poll.sh
#   bash scripts/aos-gzmo-poll.sh --check-http   # expects aos-status-feed --serve already up
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
CHECK_HTTP=0
for a in "$@"; do
  case "$a" in
    --check-http) CHECK_HTTP=1 ;;
  esac
done

bash "$ROOT/scripts/aos-status-feed.sh"

FILE="$DATA/aos-status/latest.json"
if [[ ! -f "$FILE" ]]; then
  echo "[!] missing $FILE" >&2
  exit 1
fi

python3 - <<PY
import json, sys
from pathlib import Path
p = Path("$FILE")
d = json.loads(p.read_text(encoding="utf-8"))
need = ("status", "z_score", "gzmo")
missing = [k for k in need if k not in d]
if missing:
    print(f"[!] telemetry missing keys: {missing}", file=sys.stderr)
    sys.exit(1)
g = d.get("gzmo") or {}
print(json.dumps({
    "ok": True,
    "poll": "file",
    "path": str(p),
    "status": d.get("status"),
    "z_score": d.get("z_score"),
    "euro_night_total": g.get("euro_night_total"),
    "concept_gate": g.get("concept_gate"),
    "watchdog_stale": g.get("watchdog_stale"),
}, indent=2))
PY

if [[ "$CHECK_HTTP" -eq 1 ]]; then
  url="${GZMO_AOS_STATUS_URL:-http://127.0.0.1:8765/telemetry.json}"
  if curl -fsS --max-time 2 "$url" >/tmp/gzmo-aos-telemetry.json; then
    python3 - <<'PY'
import json
d = json.load(open("/tmp/gzmo-aos-telemetry.json", encoding="utf-8"))
assert "status" in d and "gzmo" in d
print(json.dumps({"ok": True, "poll": "http", "url": "http://127.0.0.1:8765/telemetry.json", "z_score": d.get("z_score")}, indent=2))
PY
  else
    echo "[!] HTTP poll failed at $url — start: bash scripts/aos-status-feed.sh --serve" >&2
    exit 1
  fi
fi
