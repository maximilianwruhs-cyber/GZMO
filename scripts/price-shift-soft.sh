#!/usr/bin/env bash
# Soft price-shift note from price-window suggestions (no cron mutate).
# Writes data-next/scheduler-runs/latest-price-shift.json
#
#   bash scripts/price-shift-soft.sh
#   GZMO_PRICE_SHIFT=1 …   # serve may delay distill/dream until suggested UTC (opt-in)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
export DATA
export PRICE_SHIFT_OPT_IN="${GZMO_PRICE_SHIFT:-0}"

# Refresh suggestions if missing/stale-ish
if [[ ! -f "$DATA/price-window/latest.json" ]]; then
  bash "$ROOT/scripts/price-window-suggest.sh" || true
fi

python3 - <<'PY'
import json, os
from datetime import datetime, timezone
from pathlib import Path

data = Path(os.environ["DATA"])
pw_path = data / "price-window" / "latest.json"
out_dir = data / "scheduler-runs"
out_dir.mkdir(parents=True, exist_ok=True)
opt_in = os.environ.get("PRICE_SHIFT_OPT_IN", "0").strip().lower() in ("1", "true", "yes", "on")

now = datetime.now(timezone.utc)
try:
    pw = json.loads(pw_path.read_text(encoding="utf-8"))
except Exception as e:
    payload = {
        "schema": "gzmo.price.shift/v1",
        "generated_at": now.isoformat(),
        "ok": False,
        "opt_in": opt_in,
        "detail": f"no price-window: {e}",
        "actions": [],
    }
    (out_dir / "latest-price-shift.json").write_text(json.dumps(payload, indent=2) + "\n")
    print(json.dumps(payload, indent=2))
    raise SystemExit(0)

actions = []
for job, sug in (pw.get("suggestions") or {}).items():
    start = sug.get("suggested_start_utc")
    nominal = sug.get("nominal_utc")
    shift_h = sug.get("shift_hours")
    savings = sug.get("savings_c_kwh")
    if start:
        note = (
            f"would shift {job} from {nominal} → {start} "
            f"(Δ{shift_h}h, save {savings} ¢/kWh); metabolism still wins"
        )
    else:
        note = sug.get("note") or f"no shift advice for {job} (keep nominal {nominal})"
    actions.append({
        "job": job,
        "nominal_utc": nominal,
        "suggested_start_utc": start,
        "shift_hours": shift_h,
        "savings_c_kwh": savings,
        "suggested_c_kwh": sug.get("suggested_c_kwh"),
        "note": note,
        "apply": "delay_until_suggested" if opt_in else "log_only",
    })

payload = {
    "schema": "gzmo.price.shift/v1",
    "generated_at": now.isoformat(),
    "ok": True,
    "opt_in": opt_in,
    "live": pw.get("live"),
    "cheapest_24h": pw.get("cheapest_24h"),
    "actions": actions,
    "note": "Sibling soft advice — cron not overwritten. Set GZMO_PRICE_SHIFT=1 for serve soft-delay.",
}
(out_dir / "latest-price-shift.json").write_text(json.dumps(payload, indent=2) + "\n")
print(json.dumps({"ok": True, "opt_in": opt_in, "actions": len(actions), "path": str(out_dir / "latest-price-shift.json")}, indent=2))
for a in actions:
    print(f"  - {a['note']}")
PY
