#!/usr/bin/env bash
# Soft price-aware overnight window suggestion (Awattar AT).
# Does NOT mutate serve cron — writes a sibling suggestion only.
#
#   bash scripts/price-window-suggest.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/price-window"
mkdir -p "$OUT"
export OUT

# Nominal UTC anchors (serve soft jobs); override via env.
export NOMINAL_DISTILL_UTC="${NOMINAL_DISTILL_UTC:-02:00}"
export NOMINAL_DREAM_UTC="${NOMINAL_DREAM_UTC:-03:00}"
export WINDOW_HOURS="${PRICE_WINDOW_HOURS:-2}"

python3 - <<'PY'
import json, os, urllib.request
from datetime import datetime, timezone, timedelta
from pathlib import Path

out = Path(os.environ["OUT"])
window_h = int(os.environ.get("WINDOW_HOURS", "2"))
nominal = {
    "distill": os.environ.get("NOMINAL_DISTILL_UTC", "02:00"),
    "dream": os.environ.get("NOMINAL_DREAM_UTC", "03:00"),
}


def parse_hhmm(s: str) -> tuple[int, int]:
    hh, mm = s.split(":")
    return int(hh), int(mm)


def fetch_awattar():
    with urllib.request.urlopen("https://api.awattar.at/v1/marketdata", timeout=8) as resp:
        return json.loads(resp.read().decode()).get("data") or []


now = datetime.now(timezone.utc)
try:
    slots = fetch_awattar()
    live = True
except Exception as e:
    slots = []
    live = False
    err = str(e)
else:
    err = None

# Normalize slots: start/end unix ms → ¢/kWh (marketprice is €/MWh → /10 = ¢/kWh)
norm = []
for e in slots:
    try:
        start = datetime.fromtimestamp(e["start_timestamp"] / 1000.0, tz=timezone.utc)
        end = datetime.fromtimestamp(e["end_timestamp"] / 1000.0, tz=timezone.utc)
        c_kwh = float(e["marketprice"]) / 10.0
        norm.append({"start": start, "end": end, "c_kwh": c_kwh})
    except Exception:
        continue

suggestions = {}
for job, hhmm in nominal.items():
    hh, mm = parse_hhmm(hhmm)
    # Next occurrence of nominal today/tomorrow UTC
    cand = now.replace(hour=hh, minute=mm, second=0, microsecond=0)
    if cand < now - timedelta(hours=1):
        cand += timedelta(days=1)
    lo = cand - timedelta(hours=window_h)
    hi = cand + timedelta(hours=window_h)
    in_win = [s for s in norm if s["end"] > lo and s["start"] < hi]
    if in_win:
        best = min(in_win, key=lambda s: s["c_kwh"])
        nominal_slot = min(
            in_win,
            key=lambda s: abs((s["start"] - cand).total_seconds()),
        )
        suggestions[job] = {
            "nominal_utc": cand.isoformat(),
            "window_utc": [lo.isoformat(), hi.isoformat()],
            "suggested_start_utc": best["start"].isoformat(),
            "suggested_c_kwh": round(best["c_kwh"], 4),
            "nominal_c_kwh": round(nominal_slot["c_kwh"], 4),
            "savings_c_kwh": round(nominal_slot["c_kwh"] - best["c_kwh"], 4),
            "shift_hours": round((best["start"] - cand).total_seconds() / 3600.0, 2),
        }
    else:
        suggestions[job] = {
            "nominal_utc": cand.isoformat(),
            "window_utc": [lo.isoformat(), hi.isoformat()],
            "suggested_start_utc": None,
            "note": "no Awattar slots in window" if live else "awattar fetch failed",
        }

cheapest = min(norm, key=lambda s: s["c_kwh"]) if norm else None
payload = {
    "schema": "gzmo.price.window/v1",
    "generated_at": now.isoformat(),
    "live": live,
    "error": err,
    "window_hours": window_h,
    "nominal_utc": nominal,
    "suggestions": suggestions,
    "cheapest_24h": (
        {
            "start": cheapest["start"].isoformat(),
            "c_kwh": round(cheapest["c_kwh"], 4),
        }
        if cheapest
        else None
    ),
    "note": "Sibling suggestion only — do not auto-shift serve cron; metabolism still wins.",
}

(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
lines = [
    "# Price-aware overnight window",
    "",
    f"Generated: {payload['generated_at']} · live={live}",
    "",
]
for job, s in suggestions.items():
    lines.append(
        f"- **{job}**: nominal {s.get('nominal_utc')} → suggest {s.get('suggested_start_utc')} "
        f"({s.get('suggested_c_kwh')} ¢/kWh, Δ={s.get('savings_c_kwh')} ¢, shift={s.get('shift_hours')}h)"
    )
lines += ["", payload["note"], ""]
(out / "latest.md").write_text("\n".join(lines), encoding="utf-8")
print(json.dumps({"live": live, "suggestions": {k: v.get("suggested_start_utc") for k, v in suggestions.items()}, "cheapest_24h": payload["cheapest_24h"]}, indent=2))
print(f"Wrote {out / 'latest.json'}")
PY
