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

# Run constituent checks (soft — collect artifacts)
CHECKS=(
  herdr-metabolism-check.sh
  pi-glass-check.sh
  tinyfolder-check.sh
  aos-poll-check.sh
  pantheon-ritual-check.sh
  discovery-theater-check.sh
  hsp-emit-check.sh
  arena-lab-check.sh
  ipw-route-check.sh
  forge-lab-check.sh
)

for c in "${CHECKS[@]}"; do
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
    ("aos_ce_doc", root/"docs"/"AOS_CUSTOMER_EDITION.md"),
    ("marketplace_doc", root/"docs"/"OKCP_MARKETPLACE.md"),
    ("wiki_mind_doc", root/"docs"/"WIKI_OBSERVATORY_MIND.md"),
    ("portable_rfc", root/"docs"/"PORTABLE_GZMO_CORE_RFC.md"),
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
            st = "PASS" if d.get("ok") else "FAIL"
            if d.get("ok") and d.get("counts",{}).get("hold",0) > 0:
                st = "HOLD" if "ok" in d.get("advice","") or "hold" in d.get("advice","").lower() else st
            # Prefer advice-based: ok with holds → HOLD row for wave visibility
            if d.get("ok") and int(d.get("counts",{}).get("hold",0)) > 0:
                st = "HOLD"
            advice = d.get("advice","")
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
