#!/usr/bin/env bash
# O8 — Arena night dry ritual (suggest-only). Prefer local; avoid Actions.
#   ARENA_DRY=1 bash scripts/arena-night-check.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/arena"
mkdir -p "$OUT"

# Prefer existing latest.json honesty when full night is too heavy for minute budget.
if [[ "${ARENA_FORCE_RUN:-0}" != "1" && -f "$OUT/latest.json" ]]; then
  python3 - "$OUT" <<'PY'
import json, sys
from datetime import datetime, timezone
from pathlib import Path
out = Path(sys.argv[1])
d = json.loads((out / "latest.json").read_text())
auto = d.get("auto_apply")
if auto is True:
    print(json.dumps({"ok": False, "error": "auto_apply_true_forbidden"})); raise SystemExit(1)
champ = out / "champion-suggestion.toml"
# sibling may be named differently
siblings = list(out.glob("*champion*.toml")) + list(out.glob("*suggestion*.toml"))
payload = {
    "schema": "gzmo.arena_night.check/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "ok": True,
    "mode": "artifact_review",
    "auto_apply": False,
    "latest": str(out / "latest.json"),
    "suggestion_siblings": [str(p) for p in siblings],
    "advice": "arena_night_ok — suggest-only artifact present (ARENA_FORCE_RUN=1 for fresh burst)",
}
(out / "check-latest.json").write_text(json.dumps(payload, indent=2) + "\n")
print(json.dumps(payload, indent=2))
PY
  exit 0
fi

echo "=== arena-night (forced) ==="
bash "$ROOT/scripts/arena-night.sh" 2>&1 | tee "$OUT/check-run.log" | tail -20
python3 - "$OUT" <<'PY'
import json, sys
from datetime import datetime, timezone
from pathlib import Path
out = Path(sys.argv[1])
d = json.loads((out / "latest.json").read_text())
assert d.get("auto_apply") is not True
payload = {
    "schema": "gzmo.arena_night.check/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "ok": True,
    "mode": "fresh_run",
    "auto_apply": False,
    "advice": "arena_night_ok — fresh suggest-only burst",
}
(out / "check-latest.json").write_text(json.dumps(payload, indent=2) + "\n")
print(json.dumps(payload, indent=2))
PY
