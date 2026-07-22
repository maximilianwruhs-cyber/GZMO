#!/usr/bin/env bash
# O7 — REM substrate in dream recipe (Experience A). Local-first; no Actions.
#   bash scripts/rem-dream-check.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CLONE="${GZMO_CLONE_ROOT:-$(dirname "$ROOT")}"
LAB="${LITTLE_TOOLS_LAB_ROOT:-$CLONE/little-tools-lab}"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/rem-dream"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$CLONE/temp-bench/target}"
export GZMO_CLONE_ROOT="$CLONE"
export VAULT_PATH="${VAULT_PATH:-$DATA/vault.db}"
mkdir -p "$OUT"

DREAMS_OUT="$OUT/DREAMS-fixture.md"
STATS="$OUT/dream-stats.json"

echo "=== rem-dream: session-to-dream fixture ==="
bash "$LAB/scripts/session-to-dream.sh" --fixture \
  --stats "$STATS" \
  -o "$DREAMS_OUT" \
  --vault "$VAULT_PATH" \
  2>&1 | tee "$OUT/run.log" | tail -40

[[ -f "$DREAMS_OUT" ]] || DREAMS_OUT="$DATA/DREAMS.md"

python3 - "$STATS" "$DREAMS_OUT" "$OUT" <<'PY'
import json, sys, re
from datetime import datetime, timezone
from pathlib import Path
stats_p, dreams_p, out = Path(sys.argv[1]), Path(sys.argv[2]), Path(sys.argv[3])
stats = json.loads(stats_p.read_text()) if stats_p.is_file() else {}
dreams = dreams_p.read_text(encoding="utf-8") if dreams_p.is_file() else ""
rem_anchors = int(stats.get("rem_anchors") or 0)
has_section = "HONEYPOT ASSOCIATIONS" in dreams or "rem-substrate" in dreams.lower()
errors = []
if rem_anchors < 1:
    errors.append("rem_anchors_lt_1")
if not has_section:
    errors.append("missing_rem_section_in_dreams")
payload = {
    "schema": "gzmo.rem_dream.check/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "ok": not errors,
    "rem_anchors": rem_anchors,
    "rem_chars": stats.get("rem_chars"),
    "has_honeypot_associations_section": has_section,
    "errors": errors,
    "advice": "rem_dream_ok — Experience A REM section demable" if not errors else f"rem_dream_fail — {','.join(errors)}",
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n")
print(json.dumps(payload, indent=2))
raise SystemExit(0 if payload["ok"] else 1)
PY
