#!/usr/bin/env bash
# Cognis dialect weekend stub — confidence-gated plan checks over plan-gate fixtures.
# Explicitly NOT a production brain; soft-fail; never on GREEN overnight.
#
#   bash scripts/cognis-dialect-stub.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CLONE="${GZMO_CLONE_ROOT:-$(dirname "$ROOT")}"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/cognis-dialect"
PG="${PLAN_GATE_ROOT:-$CLONE/plan-gate}"
TARGET="${CARGO_TARGET_DIR:-$HOME/github-clone/temp-bench/target}"
BIN="${PLAN_GATE_BIN:-$TARGET/release/plan-gate}"
mkdir -p "$OUT"

# Tiny dialect program (not executed by serve).
cat >"$OUT/dialect.json" <<'EOF'
{
  "schema": "gzmo.cognis.dialect/v0",
  "name": "cognis-lite",
  "production": false,
  "statements": [
    {
      "id": "plan.accept",
      "form": "ACCEPT plan WHEN plan-gate PASS AND confidence >= 0.7",
      "tool": "plan-gate check",
      "confidence_min": 0.7
    },
    {
      "id": "plan.hold",
      "form": "HOLD plan WHEN plan-gate FAIL OR confidence < 0.7",
      "tool": "plan-gate check",
      "confidence_min": 0.7
    },
    {
      "id": "brain.ban",
      "form": "NEVER route chat/serve through Cognis dialect",
      "tool": null,
      "confidence_min": 1.0
    }
  ],
  "note": "Weekend prototype surface — typed confidence gates over existing tools only."
}
EOF

STATUS="fixture_skip"
EXIT=0
PLAN=""
if [[ -x "$BIN" ]]; then
  if [[ -f "$PG/fixtures/valid-plan.json" ]]; then
    PLAN="$PG/fixtures/valid-plan.json"
    if "$BIN" check --plan "$PLAN" >"$OUT/plan-gate-stdout.txt" 2>"$OUT/plan-gate-stderr.txt"; then
      STATUS="accept_demo"
    else
      EXIT=$?
      STATUS="hold_demo"
    fi
  else
    STATUS="bin_no_fixture"
  fi
elif [[ -f "$PG/Cargo.toml" ]]; then
  PLAN="$PG/fixtures/valid-plan.json"
  if [[ -f "$PLAN" ]] && (
    cd "$PG"
    export CARGO_TARGET_DIR="$TARGET"
    cargo run --release --quiet -- check --plan "$PLAN"
  ) >"$OUT/plan-gate-stdout.txt" 2>"$OUT/plan-gate-stderr.txt"; then
    STATUS="accept_demo"
    BIN="$TARGET/release/plan-gate"
  else
    EXIT=$?
    STATUS="cargo_soft_fail"
  fi
else
  STATUS="plan_gate_missing"
fi

# Confidence stub: fixture demos are high confidence; missing tools → hold.
CONF="0.85"
if [[ "$STATUS" != "accept_demo" ]]; then
  CONF="0.4"
fi
VERDICT="ACCEPT"
if awk "BEGIN {exit !($CONF >= 0.7)}" && [[ "$STATUS" == "accept_demo" ]]; then
  VERDICT="ACCEPT"
else
  VERDICT="HOLD"
fi

export OUT STATUS EXIT PLAN BIN CONF VERDICT
python3 - <<'PY'
import json, os
from datetime import datetime, timezone
from pathlib import Path

out = Path(os.environ["OUT"])
now = datetime.now(timezone.utc).isoformat()
status = os.environ["STATUS"]
verdict = os.environ["VERDICT"]
conf = float(os.environ["CONF"])
payload = {
    "schema": "gzmo.cognis.dialect.run/v0",
    "generated_at": now,
    "ok": True,  # soft research stub
    "production_brain": False,
    "status": status,
    "verdict": verdict,
    "confidence": conf,
    "plan": os.environ.get("PLAN") or None,
    "plan_gate_bin": os.environ.get("BIN"),
    "plan_gate_exit": int(os.environ.get("EXIT") or 0),
    "dialect": str(out / "dialect.json"),
    "advice": "research_only — do not wire into serve/chat; Cognis is not the production brain",
    "note": "Weekend stub over plan-gate fixtures; no production coupling.",
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
(out / "latest.md").write_text(
    "\n".join(
        [
            "# Cognis dialect stub",
            "",
            f"Verdict: **{verdict}** (confidence={conf})",
            f"Status: {status}",
            "",
            payload["advice"],
            "",
            payload["note"],
            "",
        ]
    ),
    encoding="utf-8",
)
print(json.dumps({"ok": True, "verdict": verdict, "confidence": conf, "status": status}, indent=2))
PY
exit 0
