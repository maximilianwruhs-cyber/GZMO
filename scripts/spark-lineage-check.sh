#!/usr/bin/env bash
# O6 / Experience B — spark lineage operator surface proof.
#   bash scripts/spark-lineage-check.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CLONE="${GZMO_CLONE_ROOT:-$(dirname "$ROOT")}"
LAB="${LITTLE_TOOLS_LAB_ROOT:-$CLONE/little-tools-lab}"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/spark"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$CLONE/temp-bench/target}"
export GZMO_CLONE_ROOT="$CLONE"
export VAULT_PATH="${VAULT_PATH:-$DATA/vault.db}"
export SPARK_PERSIST_DIR="$OUT"

mkdir -p "$OUT"

echo "=== spark-lineage: cognition-smoke fixture (persist) ==="
bash "$LAB/scripts/cognition-smoke.sh" --fixture --meta "$OUT/check-cognition-meta.json"

REPORT="$OUT/last-spark-report.json"
CARD_MD="$OUT/latest-card.md"
CARD_JSON="$OUT/lineage-latest.json"

[[ -f "$REPORT" ]] || { echo "missing $REPORT" >&2; exit 1; }
[[ -f "$CARD_MD" ]] || { echo "missing $CARD_MD" >&2; exit 1; }
[[ -f "$CARD_JSON" ]] || { echo "missing $CARD_JSON" >&2; exit 1; }

export ROOT OUT
python3 - <<'PY'
import json, os, subprocess
from pathlib import Path

out = Path(os.environ["OUT"])
report = json.loads((out / "last-spark-report.json").read_text(encoding="utf-8"))
card = json.loads((out / "lineage-latest.json").read_text(encoding="utf-8"))
md = (out / "latest-card.md").read_text(encoding="utf-8")
sel = report.get("selection") or {}
anchor = (sel.get("anchor") or {}).get("content") or report.get("anchor_preview")
stale = sel.get("stale_sweetness")
if stale is None:
    stale = card.get("stale_sweetness")

errors = []
if not anchor:
    errors.append("no_anchor")
if stale is None or float(stale) <= 0:
    errors.append("stale_sweetness_missing_or_zero")
if not card.get("experience_b_ok"):
    errors.append("experience_b_ok_false")
if "stale_sweetness" not in md and "Last spark" not in md:
    errors.append("card_md_thin")
if card.get("schema") != "gzmo.spark.lineage_card/v1":
    errors.append("bad_card_schema")

# Unit goldens for lineage parser
root = Path(os.environ["ROOT"])
r = subprocess.run(
    ["cargo", "test", "-p", "gzmo-core", "spark_lineage::tests", "--", "--quiet"],
    cwd=str(root),
    capture_output=True,
    text=True,
)
if r.returncode != 0:
    errors.append("spark_lineage_unit_fail")
    print(r.stdout)
    print(r.stderr)

payload = {
    "schema": "gzmo.spark_lineage.check/v1",
    "ok": not errors,
    "experience_b_ok": bool(card.get("experience_b_ok")),
    "stale_sweetness": stale,
    "anchor_preview": (anchor or "")[:160],
    "errors": errors,
    "advice": (
        "spark_lineage_ok — Experience B surface demable"
        if not errors
        else f"spark_lineage_fail — {','.join(errors)}"
    ),
    "artifacts": {
        "report": str(out / "last-spark-report.json"),
        "card_md": str(out / "latest-card.md"),
        "card_json": str(out / "lineage-latest.json"),
    },
}
(out / "lineage-check-latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
print(json.dumps(payload, indent=2))
raise SystemExit(0 if payload["ok"] else 1)
PY
