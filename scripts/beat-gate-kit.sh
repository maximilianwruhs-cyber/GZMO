#!/usr/bin/env bash
# Beat-gate open eval kit spike — fixture→meta→gate ladder for organ promotion.
# Wraps little-tools-lab beat-gate; writes portable kit status under data-next.
#
#   bash scripts/beat-gate-kit.sh
#   bash scripts/beat-gate-kit.sh --loops config,cognition,pedagogy
set -euo pipefail

ROOT_GZMO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CLONE="${GZMO_CLONE_ROOT:-$(dirname "$ROOT_GZMO")}"
LAB="${LITTLE_TOOLS_LAB_ROOT:-$CLONE/little-tools-lab}"
DATA="${GZMO_DATA_NEXT:-$ROOT_GZMO/data-next}"
OUT="$DATA/beat-gate"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$CLONE/temp-bench/target}"
export GZMO_CLONE_ROOT="$CLONE"
export VAULT_PATH="${VAULT_PATH:-$DATA/vault.db}"

LOOPS_CSV="config,cognition,pedagogy"
while [[ $# -gt 0 ]]; do
  case "$1" in
    --loops) LOOPS_CSV="${2:-$LOOPS_CSV}"; shift 2 ;;
    --all) LOOPS_CSV="config,ops,cognition,knowledge,discovery,pedagogy,ingest,kg"; shift ;;
    *) shift ;;
  esac
done

mkdir -p "$OUT/metas"
GATE="$LAB/scripts/beat-gate.sh"
if [[ ! -x "$GATE" && ! -f "$GATE" ]]; then
  echo "[!] missing $GATE — need little-tools-lab sibling" >&2
  exit 1
fi

IFS=',' read -r -a LOOPS <<<"$LOOPS_CSV"
pass=0
fail=0
results=()

for loop in "${LOOPS[@]}"; do
  loop="$(echo "$loop" | xargs)"
  [[ -n "$loop" ]] || continue
  meta="$OUT/metas/${loop}.json"
  echo "=== beat-gate kit: $loop (fixture) ==="
  if bash "$GATE" --loop "$loop" --fixture --meta "$meta"; then
    echo "PASS $loop"
    pass=$((pass + 1))
    results+=("PASS:$loop")
  else
    echo "FAIL $loop"
    fail=$((fail + 1))
    results+=("FAIL:$loop")
  fi
done

export OUT PASS_N="$pass" FAIL_N="$fail"
export RESULTS_CSV
RESULTS_CSV="$(IFS=','; echo "${results[*]}")"

python3 - <<'PY'
import json, os
from datetime import datetime, timezone
from pathlib import Path

out = Path(os.environ["OUT"])
pass_n = int(os.environ["PASS_N"])
fail_n = int(os.environ["FAIL_N"])
rows = []
for part in (os.environ.get("RESULTS_CSV") or "").split(","):
    if not part or ":" not in part:
        continue
    status, loop = part.split(":", 1)
    meta_path = out / "metas" / f"{loop}.json"
    meta = {}
    if meta_path.is_file():
        try:
            meta = json.loads(meta_path.read_text(encoding="utf-8"))
        except Exception:
            meta = {}
    metrics = meta.get("metrics") or {}
    gate_passed = metrics.get("gate_passed")
    if gate_passed is None:
        gate_passed = meta.get("beats_incumbent")
    rows.append({
        "loop": loop,
        "status": status,
        "meta": str(meta_path) if meta_path.is_file() else None,
        "beats_incumbent": meta.get("beats_incumbent"),
        "gate_passed": gate_passed,
        "baseline_id": meta.get("baseline_id"),
    })

# Honesty: core loops must emit boolean gate_passed (not null) when they PASS.
core = {"config", "cognition", "knowledge", "discovery", "ops"}
honesty_fail = [
    r["loop"]
    for r in rows
    if r["loop"] in core
    and r["status"] == "PASS"
    and not isinstance(r.get("gate_passed"), bool)
]
baseline_missing = [
    r["loop"]
    for r in rows
    if r["loop"] in core
    and r["status"] == "PASS"
    and not r.get("baseline_id")
]

kit_ok = fail_n == 0 and pass_n > 0 and not honesty_fail and not baseline_missing
kit = {
    "schema": "gzmo.beat-gate.kit/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "ok": kit_ok,
    "pass": pass_n,
    "fail": fail_n,
    "loops": rows,
    "honesty": {
        "null_gate_passed": honesty_fail,
        "missing_baseline_id": baseline_missing,
    },
    "ladder": ["fixture", "meta", "gate", "promote(S0→S3 human)"],
    "reference": "little-tools-lab/scripts/beat-gate.sh",
    "note": "Open eval kit — versioned baselines under little-tools-lab/fixtures/beat-baselines/; no CT101 writes.",
}
(out / "latest.json").write_text(json.dumps(kit, indent=2) + "\n", encoding="utf-8")
(out / "contract.json").write_text(
    json.dumps(
        {
            "schema": "gzmo.beat-gate.contract/v1",
            "name": "beat-gate-kit",
            "steps": [
                {"id": "fixture", "desc": "Run organ recipe in fixture mode"},
                {"id": "meta", "desc": "Emit beat-meta / recipe-meta JSON"},
                {"id": "gate", "desc": "Compare lab vs incumbent metrics"},
                {"id": "promote", "desc": "Human S0→S3 promotion — never auto"},
            ],
            "loops": [r["loop"] for r in rows],
        },
        indent=2,
    )
    + "\n",
    encoding="utf-8",
)
lines = [
    "# Beat-gate kit",
    "",
    f"PASS {pass_n} · FAIL {fail_n}",
    "",
    "| loop | status | gate_passed | baseline_id |",
    "|------|--------|-------------|-------------|",
]
for r in rows:
    lines.append(
        f"| {r['loop']} | {r['status']} | {r.get('gate_passed')} | {r.get('baseline_id') or '—'} |"
    )
lines += ["", kit["note"], ""]
if honesty_fail:
    lines.append(f"HONESTY FAIL null gate_passed: {', '.join(honesty_fail)}")
if baseline_missing:
    lines.append(f"HONESTY FAIL missing baseline_id: {', '.join(baseline_missing)}")
(out / "latest.md").write_text("\n".join(lines), encoding="utf-8")
print(json.dumps({
    "ok": kit["ok"],
    "pass": pass_n,
    "fail": fail_n,
    "honesty": kit["honesty"],
    "path": str(out / "latest.json"),
}, indent=2))
raise SystemExit(0 if kit_ok else 1)
PY
