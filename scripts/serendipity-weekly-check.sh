#!/usr/bin/env bash
# O10 — weekly serendipity apply honesty (local; no Actions).
#   bash scripts/serendipity-weekly-check.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/serendipity"
CAP="${SERENDIPITY_WEEKLY_CAP:-3}"
mkdir -p "$OUT"

# Refresh dry-run + cadence (soft)
bash "$ROOT/scripts/serendipity-cadence.sh" >/dev/null 2>&1 || true

export OUT CAP
python3 - <<'PY'
import json, os
from datetime import datetime, timezone
from pathlib import Path

out = Path(os.environ["OUT"])
cap = int(os.environ["CAP"])
now = datetime.now(timezone.utc)
iso_week = now.strftime("%G-W%V")

week_applies = 0
week_log = out / "weekly-apply-log.jsonl"
rows = []
if week_log.is_file():
    for line in week_log.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        try:
            row = json.loads(line)
        except Exception:
            continue
        if row.get("iso_week") == iso_week:
            week_applies += int(row.get("applied_count") or 0)
            rows.append(row)

promote = {}
pp = out / "promote-latest.json"
if pp.is_file():
    promote = json.loads(pp.read_text(encoding="utf-8"))

candidates = int(promote.get("candidate_count") or 0)
filtered = len(promote.get("filtered_out") or [])
auto = promote.get("auto_apply")
dual = bool(promote.get("dual_writer"))

errors = []
if auto is True:
    errors.append("auto_apply_true")
if dual:
    errors.append("dual_writer")
if week_applies > cap:
    errors.append(f"over_cap_{week_applies}/{cap}")

# O10 bar: this week has ≥1 apply when USP candidates were available, OR hold with filter honesty
ok = not errors and week_applies <= cap
if week_applies == 0 and candidates > 0:
    advice = (
        f"serendipity_weekly_remind — {candidates} USP candidates; "
        f"0/{cap} applies in {iso_week}; human gate APPLY=1"
    )
    # remind is ok=True (habit not failed)
elif week_applies == 0 and candidates == 0:
    advice = f"serendipity_weekly_hold — 0 USP candidates in {iso_week} (filtered_out={filtered})"
else:
    advice = (
        f"serendipity_weekly_ok — {week_applies}/{cap} applies in {iso_week}; "
        f"candidates={candidates}; filtered_out={filtered}"
    )

payload = {
    "schema": "gzmo.serendipity.weekly_check/v1",
    "generated_at": now.isoformat(),
    "ok": ok,
    "iso_week": iso_week,
    "week_applies": week_applies,
    "weekly_cap": cap,
    "candidates": candidates,
    "filtered_out": filtered,
    "auto_apply": False,
    "dual_writer": dual,
    "errors": errors,
    "advice": advice,
    "rows": rows[-5:],
}
(out / "weekly-check-latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
md = [
    "# Serendipity weekly",
    "",
    f"Advice: **{advice}**",
    "",
    f"- ISO week: {iso_week}",
    f"- Applies: {week_applies}/{cap}",
    f"- USP candidates: {candidates} (filtered_out={filtered})",
    f"- Auto-apply: **false**",
    "",
]
(out / "weekly-check-latest.md").write_text("\n".join(md) + "\n", encoding="utf-8")
print(json.dumps({k: payload[k] for k in ("ok", "advice", "week_applies", "candidates", "filtered_out")}, indent=2))
raise SystemExit(0 if ok else 1)
PY
