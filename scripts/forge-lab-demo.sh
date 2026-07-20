#!/usr/bin/env bash
# Unpark Wave 3.3 demable: recommend-only forge pins from Arena winners.
# Never auto-blocks distill; never writes live engine config.
#
#   bash scripts/forge-lab-demo.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/forge-lab"
ARENA="$DATA/arena"
mkdir -p "$OUT"

export OUT ARENA ROOT
python3 - <<'PY'
import json
import os
from datetime import datetime, timezone
from pathlib import Path

out = Path(os.environ["OUT"])
arena = Path(os.environ["ARENA"])
forge = arena / "forge" / "latest.json"
champ_toml = arena / "champion-suggestion.toml"
latest = arena / "latest.json"

pins = []
source = "stub"
champion = None
forge_payload = None

if forge.is_file():
    try:
        forge_payload = json.loads(forge.read_text(encoding="utf-8"))
        source = "arena.forge"
        champion = forge_payload.get("champion")
        for p in forge_payload.get("pinned") or []:
            organ = p.get("engine") or p.get("organ") or "unknown"
            pins.append(
                {
                    "organ": organ,
                    "action": p.get("action") or "pin",
                    "z": p.get("z"),
                    "quality": p.get("quality"),
                    "efficiency": p.get("efficiency"),
                    "joules": p.get("joules"),
                    "finished": p.get("finished"),
                    "rank": p.get("rank"),
                    "reason": f"arena forge pin rank={p.get('rank')} z={p.get('z')}",
                }
            )
        # Surface mutation proposals as non-blocking recommend rows
        for m in forge_payload.get("mutations") or []:
            organ = m.get("engine") or "unknown"
            reasons = ",".join(m.get("reasons") or [])
            pins.append(
                {
                    "organ": organ,
                    "action": "mutate_propose",
                    "z": m.get("z"),
                    "mutation_id": m.get("mutation_id"),
                    "reasons": m.get("reasons") or [],
                    "reason": f"sibling mutation proposal ({reasons}) — human promote only",
                }
            )
    except Exception as e:
        source = f"forge_unreadable:{e}"

if not pins and champ_toml.is_file():
    source = "arena.champion-suggestion.toml"
    # Minimal TOML parse for [arena.champion] engine_label / z
    label = None
    z = None
    for line in champ_toml.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if line.startswith("engine_label"):
            label = line.split("=", 1)[1].strip().strip('"')
        elif line.startswith("z ") or line.startswith("z="):
            try:
                z = float(line.split("=", 1)[1].strip())
            except Exception:
                pass
    if label:
        champion = label
        pins.append(
            {
                "organ": label,
                "action": "pin",
                "z": z,
                "reason": "champion-suggestion.toml (manual merge only)",
            }
        )

if not pins and latest.is_file():
    try:
        night = json.loads(latest.read_text(encoding="utf-8"))
        source = "arena.latest"
        label = night.get("engine") or "unknown"
        champion = label
        pins.append(
            {
                "organ": label,
                "action": "pin",
                "z": night.get("z"),
                "quality": night.get("quality"),
                "euro_cost": night.get("euro_cost"),
                "reason": "latest nightburst snapshot — review before pin",
            }
        )
    except Exception:
        pass

if not pins:
    source = "stub"
    pins = [
        {
            "organ": "example-winner",
            "reason": "stub — no arena forge/champion artifacts yet",
        }
    ]

rec = {
    "schema": "gzmo.unpark.forge.recommend/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "wave": "3.3",
    "action": "recommend",
    "blocks_distill": False,
    "source": source,
    "champion": champion,
    "pins": pins,
    "arena_paths": {
        "forge_latest": str(forge) if forge.is_file() else None,
        "champion_suggestion": str(champ_toml) if champ_toml.is_file() else None,
        "nightburst_latest": str(latest) if latest.is_file() else None,
    },
    "note": "Display/route advice only — never auto-overwrite live config; never block distill",
}
(out / "recommend.json").write_text(json.dumps(rec, indent=2) + "\n")
(out / "recommend.md").write_text(
    "\n".join(
        [
            "# Forge recommend (Unpark Wave 3.3)",
            "",
            f"Generated: {rec['generated_at']}",
            f"Source: `{source}`",
            f"Champion: **{champion or 'n/a'}**",
            f"blocks_distill: `{rec['blocks_distill']}`",
            "",
            "| organ | action | z | reason |",
            "|-------|--------|---|--------|",
            *[
                f"| {p.get('organ')} | {p.get('action', 'pin')} | {p.get('z', '')} | {p.get('reason', '')} |"
                for p in pins
            ],
            "",
            "Human promote only. Do not wire into gzmo-daemon distill gates.",
            "",
        ]
    ),
    encoding="utf-8",
)
print(
    json.dumps(
        {
            "ok": True,
            "source": source,
            "champion": champion,
            "pins": len(pins),
            "blocks_distill": False,
            "path": str(out / "recommend.json"),
        },
        indent=2,
    )
)
PY

bash "$ROOT/scripts/forge-lab-check.sh"
echo "[OK] Forge recommend → $OUT/recommend.json"
