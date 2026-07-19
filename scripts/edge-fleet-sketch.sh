#!/usr/bin/env bash
# Edge fleet + shared forge sketch — topology only; no multi-node sync.
# After single-node AOS is boring; metabolism stays on-box.
#
#   bash scripts/edge-fleet-sketch.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CLONE="${GZMO_CLONE_ROOT:-$(dirname "$ROOT")}"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/edge-fleet"
mkdir -p "$OUT"
export ROOT CLONE DATA OUT

python3 - <<'PY'
import json, os, subprocess
from datetime import datetime, timezone
from pathlib import Path

clone = Path(os.environ["CLONE"])
out = Path(os.environ["OUT"])
data = Path(os.environ["DATA"])
now = datetime.now(timezone.utc).isoformat()


def present(name: str) -> bool:
    p = clone / name
    return p.is_dir()


def load(p: Path):
    try:
        return json.loads(p.read_text(encoding="utf-8"))
    except Exception:
        return None


aos_ce = load(data / "aos-ce" / "latest.json") or {}
wiki = load(data / "wiki-push-latest.json") or {}

topology = {
    "hub": {
        "role": "okforge_hub",
        "present_sibling": present("okforge") or present("OKForge"),
        "wiki_repo_hint": "gzmo/gzmo-next-memory",
        "wiki_sha": (wiki.get("commit_sha") or "")[:12] or None,
    },
    "edges": [
        {
            "id": "workstation-next",
            "role": "metabolism_edge",
            "vault": "local data-next (lab)",
            "forge": "pull concepts / push via gate",
            "present": True,
        },
        {
            "id": "edge-node-sibling",
            "role": "future_edge",
            "sibling": "edge-node",
            "present": present("edge-node"),
        },
    ],
    "rules": [
        "Local metabolism stays on-box — no remote vault owner swap in this spike.",
        "Shared forge is hub-only; edges publish through concept-gate.",
        "Do not arm multi-node sync until AOS CE single-node is boring.",
    ],
}

advice = (
    "hold_fleet — pin AOS CE single-node first"
    if (aos_ce.get("advice") or "").startswith("ready_to_pin")
    else "hold_fleet — need CE pin + boring single-node before edges"
)

payload = {
    "schema": "gzmo.edge-fleet.sketch/v1",
    "generated_at": now,
    "ok": True,
    "topology": topology,
    "aos_ce_advice": aos_ce.get("advice"),
    "advice": advice,
    "note": "Sketch only — no daemon, no sync, no CT101 vault takeover.",
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
(out / "latest.md").write_text(
    "\n".join(
        [
            "# Edge fleet sketch",
            "",
            f"Advice: **{advice}**",
            f"Hub wiki sha: {topology['hub']['wiki_sha'] or 'n/a'}",
            f"edge-node sibling: {topology['edges'][1]['present']}",
            "",
            payload["note"],
            "",
        ]
    ),
    encoding="utf-8",
)
print(json.dumps({"ok": True, "advice": advice, "hub": topology["hub"]}, indent=2))
PY
