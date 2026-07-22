#!/usr/bin/env bash
# Organism surface card (G10) — spark lineage + ripen + CORE_INSIGHT + Felt Use.
# Theater stays demable; this is the primary living UX aggregation.
#
#   bash scripts/organism-surface.sh
# Artifact: data-next/organism-surface/latest.{json,md,html}
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/organism-surface"
mkdir -p "$OUT"

# Soft-run satellites (never fail the whole surface on one miss)
bash "$ROOT/scripts/spark-lineage-check.sh" >/tmp/organism-spark.txt 2>&1 || true
bash "$ROOT/scripts/felt-use-depth.sh" >/tmp/organism-felt.txt 2>&1 || true
bash "$ROOT/scripts/honeypot-lifecycle-check.sh" >/tmp/organism-life.txt 2>&1 || true
bash "$ROOT/scripts/faithfulness-living.sh" >/tmp/organism-faith.txt 2>&1 || true

export OUT ROOT DATA
python3 - <<'PY'
import json, os
from datetime import datetime, timezone
from pathlib import Path

root = Path(os.environ["ROOT"])
data = Path(os.environ["DATA"])
out = Path(os.environ["OUT"])
now = datetime.now(timezone.utc).isoformat()

def load(p: Path):
    try:
        return json.loads(p.read_text(encoding="utf-8"))
    except Exception:
        return None

parts = {
    "spark_lineage": load(data / "spark" / "lineage-latest.json"),
    "felt_use": load(data / "felt-use-depth" / "latest.json"),
    "lifecycle": load(data / "honeypot-lifecycle" / "latest.json"),
    "faithfulness": load(data / "faithfulness-living" / "latest.json"),
    "core_insight_doc": (root / "docs" / "CORE_INSIGHT.md").exists(),
}
ok_n = sum(1 for v in parts.values() if v)
payload = {
    "schema": "gzmo.organism_surface/v1",
    "generated_at": now,
    "parts": {k: (True if v else False) if not isinstance(v, dict) else {"present": True, "advice": v.get("advice") or v.get("verdict")} for k, v in parts.items()},
    "ok": ok_n >= 3,
    "advice": "organism_surface_ok — reflection loop visible" if ok_n >= 3 else "organism_surface_thin — run spark/felt/lifecycle/faithfulness",
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
lines = ["# Organism surface", "", f"generated_at={now}", f"advice={payload['advice']}", ""]
for k, v in parts.items():
    lines.append(f"- {k}: {'OK' if v else 'MISS'}")
(out / "latest.md").write_text("\n".join(lines) + "\n", encoding="utf-8")
html = f"""<!doctype html><meta charset=utf-8><title>GZMO organism</title>
<style>body{{font-family:Georgia,serif;max-width:42rem;margin:2rem auto;padding:0 1rem;background:#0f1410;color:#e8efe6}}
a{{color:#9fd4a3}} h1{{font-weight:600}}</style>
<h1>GZMO organism</h1>
<p>{payload['advice']}</p>
<ul>
<li>Spark lineage</li><li>Felt Use depth</li><li>Honeypot ripen / lifecycle</li>
<li>CORE_INSIGHT self-model</li>
</ul>
<p><small>{now}</small></p>
"""
(out / "latest.html").write_text(html, encoding="utf-8")
print(json.dumps({"ok": payload["ok"], "advice": payload["advice"]}, indent=2))
PY
