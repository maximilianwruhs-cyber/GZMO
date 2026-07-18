#!/usr/bin/env bash
# Aggregate Arena (+ optional metabolism duration estimate) into €/night.
#   bash scripts/euro-night-aggregate.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
export DATA

python3 - <<'PY'
import json, os, time
from datetime import datetime, timezone
from pathlib import Path

data = Path(os.environ["DATA"])
arena = data / "arena"
hist = arena / "history"
hist.mkdir(parents=True, exist_ok=True)
latest = arena / "latest.json"

# Seed history from latest if empty.
if latest.exists() and not any(hist.glob("arena-*.json")):
    try:
        payload = json.loads(latest.read_text(encoding="utf-8"))
        fin = (payload.get("finished") or "").replace(":", "").replace("-", "")[:15] or "seed"
        (hist / f"arena-{fin}.json").write_text(
            json.dumps(payload, indent=2) + "\n", encoding="utf-8"
        )
    except Exception:
        pass

runs = []
for path in sorted(hist.glob("arena-*.json")):
    try:
        r = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        continue
    runs.append(
        {
            "file": path.name,
            "finished": r.get("finished"),
            "euro_cost": r.get("euro_cost"),
            "joules": r.get("joules"),
            "z": r.get("z"),
            "electricity_c_kwh": r.get("electricity_c_kwh"),
            "energy_source": r.get("energy_source"),
        }
    )

euros = [float(r["euro_cost"]) for r in runs if r.get("euro_cost") is not None]
joules = [float(r["joules"]) for r in runs if r.get("joules") is not None]

# Metabolism duration estimate from latest-* finished windows (soft, labeled).
# Sum elapsed-like fields if present; else skip.
metab_ms = 0
metab_jobs = []
runs_dir = data / "scheduler-runs"
for job in ("distill", "promote", "embed", "dream", "spark"):
    p = runs_dir / f"latest-{job}.json"
    if not p.exists():
        continue
    try:
        j = json.loads(p.read_text(encoding="utf-8"))
    except Exception:
        continue
    ms = j.get("elapsed_ms") or j.get("duration_ms")
    if ms is None:
        continue
    metab_ms += int(ms)
    metab_jobs.append({"job": job, "ms": int(ms)})

# Estimate metab € using latest arena price + 65W default if we have a price.
price_c = None
if runs:
    price_c = runs[-1].get("electricity_c_kwh")
if price_c is None and latest.exists():
    try:
        price_c = json.loads(latest.read_text(encoding="utf-8")).get("electricity_c_kwh")
    except Exception:
        price_c = None

metab_euro = None
metab_joules = None
if metab_ms > 0 and price_c is not None:
    watts = float(os.environ.get("METAB_WATTS_ESTIMATE", "65"))
    metab_joules = watts * (metab_ms / 1000.0)
    # € = joules/3600/1000 * (c/kWh)/100
    metab_euro = (metab_joules / 3_600_000.0) * (float(price_c) / 100.0)

arena_euro = sum(euros) if euros else 0.0
total_euro = arena_euro + (metab_euro or 0.0)

out = {
    "schema": "gzmo.euro.night/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "arena_runs": len(runs),
    "arena_euro_sum": round(arena_euro, 8) if euros else None,
    "arena_joules_sum": round(sum(joules), 3) if joules else None,
    "metabolism_ms": metab_ms or None,
    "metabolism_joules_est": round(metab_joules, 3) if metab_joules is not None else None,
    "metabolism_euro_est": round(metab_euro, 8) if metab_euro is not None else None,
    "metabolism_jobs": metab_jobs,
    "electricity_c_kwh": price_c,
    "euro_night_total": round(total_euro, 8) if (euros or metab_euro is not None) else None,
    "note": "Arena history is measured; metabolism € is duration×W estimate unless RAPL wired into serve jobs.",
    "runs": runs[-20:],
}

out_dir = arena
(out_dir / "euro-night.json").write_text(json.dumps(out, indent=2) + "\n", encoding="utf-8")
(out_dir / "euro-night.md").write_text(
    "\n".join(
        [
            "# €/night aggregate",
            "",
            f"Generated: {out['generated_at']}",
            f"Arena runs: {out['arena_runs']}",
            f"Arena € sum: {out['arena_euro_sum']}",
            f"Metabolism € est: {out['metabolism_euro_est']}",
            f"**Night total €: {out['euro_night_total']}**",
            f"Price: {out['electricity_c_kwh']} ¢/kWh",
            "",
        ]
    ),
    encoding="utf-8",
)
print(json.dumps({k: out[k] for k in (
    "arena_runs", "arena_euro_sum", "metabolism_euro_est", "euro_night_total", "electricity_c_kwh"
)}, indent=2))
print(f"Wrote {out_dir / 'euro-night.json'}")
PY
