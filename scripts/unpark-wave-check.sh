#!/usr/bin/env bash
# Aggregate Unpark wave artifact presence (Waves 1–4 checks).
#   bash scripts/unpark-wave-check.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/unpark-waves"
mkdir -p "$OUT"
LOG="$OUT/check.log"
: >"$LOG"

echo "=== Unpark wave check ===" | tee -a "$LOG"

# Run demable wave scripts then checks (soft — collect artifacts)
DEMOS=(
  herdr-metabolism-demo.sh
  pi-glass-fix.sh
  tinyfolder-ingest-demo.sh
  aos-poll-dashboard.sh
  pantheon-ritual-demo.sh
  discovery-theater-demo.sh
  hsp-emit-demo.sh
  arena-lab-demo.sh
  ipw-route-demo.sh
  forge-lab-demo.sh
  aos-ce-smoke.sh
  marketplace-check.sh
  wiki-mind-check.sh
  portable-core-inventory.sh
)

for c in "${DEMOS[@]}"; do
  echo "[*] $c" | tee -a "$LOG"
  bash "$ROOT/scripts/$c" >>"$LOG" 2>&1 || true
done

# Wave 4 docs presence
export OUT ROOT DATA
python3 - <<'PY'
import json, os
from datetime import datetime, timezone
from pathlib import Path

root = Path(os.environ["ROOT"])
data = Path(os.environ["DATA"])
out = Path(os.environ["OUT"])

waves = {
  "1": [
    ("herdr", data/"herdr-metabolism"/"latest.json"),
    ("pi_glass", data/"pi-glass"/"latest.json"),
    ("tinyfolder", data/"tinyfolder"/"latest.json"),
    ("aos_poll", data/"aos-poll"/"latest.json"),
  ],
  "2": [
    ("pantheon", data/"pantheon-ritual"/"latest.json"),
    ("discovery_theater", data/"discovery-theater"/"latest.json"),
    ("hsp_emit", data/"hsp-emit"/"latest.json"),
  ],
  "3": [
    ("arena_lab", data/"arena-lab"/"latest.json"),
    ("ipw", data/"ipw-route"/"latest.json"),
    ("forge_lab", data/"forge-lab"/"latest.json"),
  ],
  "4": [
    ("aos_ce", data/"aos-ce"/"latest.json"),
    ("marketplace", data/"marketplace"/"latest.json"),
    ("wiki_mind", data/"wiki-mind"/"latest.json"),
    ("portable_core", data/"portable-core"/"latest.json"),
    ("unpark_roadmap", root/"docs"/"UNPARK_ROADMAP.md"),
  ],
}

summary = {}
fail = 0
hold = 0
pass_n = 0
for wave, items in waves.items():
    rows = []
    for name, path in items:
        if path.suffix == ".json" and path.is_file():
            d = json.loads(path.read_text())
            advice = d.get("advice","")
            if not d.get("ok"):
                st = "FAIL"
            elif "_ok" in advice or advice.endswith("_ok") or "inventory_ok" in advice or "demo_ok" in advice:
                st = "PASS"
            elif "hold" in advice.lower():
                st = "HOLD"
            else:
                st = "PASS" if d.get("ok") else "FAIL"
            rows.append({"name": name, "status": st, "advice": advice})
            if st == "FAIL":
                fail += 1
            elif st == "HOLD":
                hold += 1
            else:
                pass_n += 1
        elif path.is_file():
            rows.append({"name": name, "status": "PASS", "advice": str(path)})
            pass_n += 1
        else:
            rows.append({"name": name, "status": "FAIL", "advice": f"missing {path}"})
            fail += 1
    summary[wave] = rows

verdict = "GREEN" if fail == 0 else "RED"
advice = "unpark_waves_ok — artifacts present" if fail == 0 else "unpark_waves_fail — missing checks/docs"
payload = {
    "schema": "gzmo.unpark.waves/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "verdict": verdict,
    "ok": fail == 0,
    "advice": advice,
    "counts": {"pass": pass_n, "fail": fail, "hold": hold},
    "waves": summary,
    "note": "HOLDs are expected for optional tools (herdr/Pi/Arena sibling).",
}
(out/"latest.json").write_text(json.dumps(payload, indent=2) + "\n")
md = ["# Unpark wave check", "", f"Verdict: **{verdict}**", ""]
for wave, rows in summary.items():
    md.append(f"## Wave {wave}")
    md.append("")
    md.append("| Status | Item | Advice |")
    md.append("|--------|------|--------|")
    for r in rows:
        md.append(f"| {r['status']} | {r['name']} | {r['advice']} |")
    md.append("")
(out/"latest.md").write_text("\n".join(md) + "\n")
print(json.dumps({"verdict": verdict, "advice": advice, "pass": pass_n, "fail": fail, "hold": hold}, indent=2))
raise SystemExit(0 if fail == 0 else 1)
PY
