#!/usr/bin/env bash
# Escape-loop / attractor brand kit — soft dry-run only; never on GREEN overnight gate.
# Keeps chaos / lorenz off the production metabolism path.
#
#   bash scripts/escape-loop-kit.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CLONE="${GZMO_CLONE_ROOT:-$(dirname "$ROOT")}"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/escape-loop"
ELB="${ESCAPE_LOOP_BENCH_ROOT:-$CLONE/escape-loop-bench}"
TARGET="${CARGO_TARGET_DIR:-$HOME/github-clone/temp-bench/target}"
BIN="${ESCAPE_LOOP_BIN:-$TARGET/release/escape-loop-bench}"
mkdir -p "$OUT"

STATUS="dry_fixture"
EXIT=0
REPORT=""
NOTE="Soft research brand spike — chaos off production; dry-run only."

if [[ -x "$BIN" ]]; then
  if "$BIN" run --generations 8 --dry-run >"$OUT/bench-stdout.txt" 2>"$OUT/bench-stderr.txt"; then
    STATUS="dry_run_ok"
  else
    EXIT=$?
    STATUS="dry_run_soft_fail"
  fi
elif [[ -f "$ELB/Cargo.toml" ]]; then
  if (
    cd "$ELB"
    export CARGO_TARGET_DIR="$TARGET"
    cargo run --release --quiet -- run --generations 8 --dry-run
  ) >"$OUT/bench-stdout.txt" 2>"$OUT/bench-stderr.txt"; then
    STATUS="cargo_dry_run_ok"
    BIN="$TARGET/release/escape-loop-bench"
  else
    EXIT=$?
    STATUS="cargo_soft_fail"
  fi
else
  STATUS="fixture_only"
  NOTE="escape-loop-bench missing; wrote brand contract only."
fi

# Prefer sibling report if present after run.
for cand in "$ELB/escape-report.json" "$OUT/escape-report.json"; do
  if [[ -f "$cand" ]]; then
    cp -f "$cand" "$OUT/escape-report.json"
    REPORT="$OUT/escape-report.json"
    break
  fi
done
if [[ -z "$REPORT" && -f "$ELB/example-escape-report.json" ]]; then
  cp -f "$ELB/example-escape-report.json" "$OUT/escape-report.json"
  REPORT="$OUT/escape-report.json"
  STATUS="${STATUS}+example_report"
fi

export OUT STATUS EXIT REPORT NOTE ELB BIN ROOT
python3 - <<'PY'
import json, os
from datetime import datetime, timezone
from pathlib import Path

out = Path(os.environ["OUT"])
status = os.environ["STATUS"]
exit_code = int(os.environ.get("EXIT") or 0)
report = os.environ.get("REPORT") or None
note = os.environ["NOTE"]
now = datetime.now(timezone.utc).isoformat()

metrics = None
if report and Path(report).is_file():
    try:
        metrics = json.loads(Path(report).read_text(encoding="utf-8"))
    except Exception:
        metrics = {"path": report, "parse": "failed"}

payload = {
    "schema": "gzmo.escape-loop.kit/v1",
    "generated_at": now,
    "ok": True,  # soft — research brand never trips nightburst GREEN math
    "status": status,
    "bench_exit": exit_code,
    "bench_bin": os.environ.get("BIN"),
    "bench_root": os.environ.get("ELB"),
    "report": report,
    "metrics_keys": list(metrics.keys())[:20] if isinstance(metrics, dict) else None,
    "production": {
        "on_green_overnight": False,
        "chaos_default": False,
        "advice": "research_brand_only — papers/benches; keep lorenz/chaos off serve path",
    },
    "note": note,
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
(out / "latest.md").write_text(
    "\n".join(
        [
            "# Escape-loop / attractor brand kit",
            "",
            f"Status: **{status}** (soft exit for nightburst)",
            f"Bench exit: {exit_code}",
            f"Report: `{report or 'none'}`",
            "",
            payload["production"]["advice"],
            "",
            note,
            "",
        ]
    ),
    encoding="utf-8",
)
print(json.dumps({"ok": True, "status": status, "bench_exit": exit_code, "report": report}, indent=2))
PY
# Always soft-success for nightburst bundle.
exit 0
