#!/usr/bin/env bash
# RAPL accessibility probe — explains Arena estimate vs measured joules.
# Does not require sudo; records whether energy_uj is readable for nightburst.
#
#   bash scripts/rapl-probe.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/rapl"
mkdir -p "$OUT"
export OUT

python3 - <<'PY'
import json, os, time
from datetime import datetime, timezone
from pathlib import Path

out = Path(os.environ["OUT"])
now = datetime.now(timezone.utc).isoformat()
paths = [
    Path("/sys/class/powercap/intel-rapl:0/energy_uj"),
    Path("/sys/class/powercap/intel-rapl:1/energy_uj"),
]
samples = []
readable = []
for p in paths:
    exists = p.exists()
    access = os.access(p, os.R_OK) if exists else False
    val = None
    err = None
    if access:
        try:
            val = int(p.read_text().strip())
            readable.append(str(p))
        except Exception as e:
            err = str(e)
            access = False
    samples.append({
        "path": str(p),
        "exists": exists,
        "readable": access,
        "energy_uj": val,
        "error": err,
    })

delta_j = None
source = "unavailable"
if readable:
    start = {p: int(Path(p).read_text().strip()) for p in readable}
    time.sleep(0.2)
    end = {p: int(Path(p).read_text().strip()) for p in readable}
    uj = sum(max(0, end[p] - start[p]) for p in readable)
    delta_j = round(uj / 1_000_000.0, 6)
    source = "rapl"
else:
    source = "estimate_required"
    # Hint: root-owned energy_uj is common on Ubuntu without caps.
advice = (
    "arena_can_measure"
    if source == "rapl"
    else "grant_read_or_cap — energy_uj root-only; Arena will keep energy_source=estimate (~65W)"
)

payload = {
    "schema": "gzmo.rapl.probe/v1",
    "generated_at": now,
    "ok": True,
    "source": source,
    "readable_paths": readable,
    "sample_delta_j_0_2s": delta_j,
    "zones": samples,
    "advice": advice,
    "note": "Probe only — does not change Arena; operator may chmod/acl or run meter with caps.",
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
(out / "latest.md").write_text(
    "\n".join(
        [
            "# RAPL probe",
            "",
            f"Source: **{source}**",
            f"Advice: {advice}",
            f"Readable: {', '.join(readable) or 'none'}",
            "",
            payload["note"],
            "",
        ]
    ),
    encoding="utf-8",
)
print(json.dumps({"ok": True, "source": source, "advice": advice, "readable": readable}, indent=2))
PY
