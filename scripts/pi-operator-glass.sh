#!/usr/bin/env bash
# Pi operator glass spike — status + Arena + wiki pulse for optional Pi frontend.
# CLI remains canonical; this is a read-only glass JSON/MD for Pi tools.
#
#   bash scripts/pi-operator-glass.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/pi-glass"
mkdir -p "$OUT"

# Refresh cheap feeds first (soft).
bash "$ROOT/scripts/aos-status-feed.sh" >/dev/null 2>&1 || true

export DATA OUT ROOT
python3 - <<'PY'
import json, os
from datetime import datetime, timezone
from pathlib import Path

data = Path(os.environ["DATA"])
out = Path(os.environ["OUT"])
now = datetime.now(timezone.utc).isoformat()


def load(p: Path):
    try:
        return json.loads(p.read_text(encoding="utf-8"))
    except Exception:
        return None


aos = load(data / "aos-status" / "latest.json") or {}
board = load(data / "arena" / "scoreboard.json") or {}
arena = load(data / "arena" / "latest.json") or {}
wiki = load(data / "wiki-push-latest.json") or {}
gate = load(data / "concept-gate" / "latest.json") or {}
euro = load(data / "arena" / "euro-night.json") or {}
wd = load(data / "scheduler-runs" / "latest-watchdog.json") or {}
ipw = load(data / "ipw-router" / "latest.json") or {}
cognition = load(data / "cognition-pack" / "latest.json") or {}

gzmo = aos.get("gzmo") or {}
glass = {
    "schema": "gzmo.pi.glass/v1",
    "generated_at": now,
    "canonical_frontend": "gzmo chat / gzmo_cli",
    "pi_role": "optional_glass",
    "status": {
        "watchdog_stale": wd.get("stale") if wd else gzmo.get("watchdog_stale"),
        "arena_z": arena.get("z") or aos.get("z_score"),
        "champion": arena.get("champion") or aos.get("current_model"),
        "euro_night_total": euro.get("euro_night_total") or gzmo.get("euro_night_total"),
        "concept_gate": gate.get("verdict") or gzmo.get("concept_gate"),
        "wiki_healthy": wiki.get("healthy"),
        "wiki_sha": (wiki.get("commit_sha") or "")[:12] or None,
        "ipw_route": ipw.get("route"),
        "cognition_pack_ok": cognition.get("ok"),
    },
    "links": {
        "scoreboard_html": str(data / "arena" / "scoreboard.html"),
        "aos_status": str(data / "aos-status" / "latest.json"),
        "observatory": "http://127.0.0.1:3000/observatory",
        "memory_bridge": "scripts/pi-gzmo-memory.sh",
    },
    "pi_hints": [
        "Use scripts/pi-gzmo-memory.sh for turn-start / search / recall — do not invent vault clients.",
        "gzmo_cli is canonical; this glass is read-only status for Pi sessions.",
        "Do not schedule GREEN metabolism jobs from Pi.",
    ],
    "note": "Pi / operator split polish spike — glass only; CLI remains authority.",
}

(out / "latest.json").write_text(json.dumps(glass, indent=2) + "\n", encoding="utf-8")
st = glass["status"]
md = [
    "# Pi operator glass",
    "",
    f"Generated: {now}",
    "",
    f"- watchdog: {'STALE' if st.get('watchdog_stale') else 'fresh'}",
    f"- arena z: {st.get('arena_z')} · champion: {st.get('champion')}",
    f"- €/night: {st.get('euro_night_total')}",
    f"- concept gate: {st.get('concept_gate')}",
    f"- wiki: healthy={st.get('wiki_healthy')} sha={st.get('wiki_sha')}",
    f"- IpW route: {st.get('ipw_route')}",
    "",
    "**Canonical frontend:** `gzmo chat`",
    "",
    glass["note"],
    "",
]
(out / "latest.md").write_text("\n".join(md), encoding="utf-8")
print(json.dumps({"ok": True, "status": st, "path": str(out / "latest.json")}, indent=2))
PY
