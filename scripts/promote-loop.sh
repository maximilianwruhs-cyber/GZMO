#!/usr/bin/env bash
# Promote-by-loop ritual (ADR-0005 / LTL ADR-0003) — never silent.
#
#   # Dry-run after kit PASS (writes promotion record only):
#   PROMOTE_LOOP=knowledge PROMOTE_ACK=1 bash scripts/promote-loop.sh
#
#   # Require living-host mutex claim when writing toward a living host:
#   bash scripts/living-host-mutex.sh claim --host workstation --note "promote knowledge"
#   PROMOTE_LOOP=knowledge PROMOTE_ACK=1 PROMOTE_APPLY=0 bash scripts/promote-loop.sh
#   bash scripts/living-host-mutex.sh release
#
# Whole-host cutover still needs CUTOVER_APPROVED=1 — this script refuses that path.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CLONE="${GZMO_CLONE_ROOT:-$(dirname "$ROOT")}"
LAB="${LITTLE_TOOLS_LAB_ROOT:-$CLONE/little-tools-lab}"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/beat-gate/promotions"
LOOP="${PROMOTE_LOOP:-}"
ACK="${PROMOTE_ACK:-}"
APPLY="${PROMOTE_APPLY:-0}"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$CLONE/temp-bench/target}"

usage() {
  echo "Usage: PROMOTE_LOOP=<config|ops|cognition|knowledge|discovery> PROMOTE_ACK=1 bash $0" >&2
  echo "  PROMOTE_APPLY=1 reserved (currently refused — record-only first ship)" >&2
  exit 2
}

[[ -n "$LOOP" ]] || usage
[[ "$ACK" == "1" ]] || {
  echo "error: set PROMOTE_ACK=1 after reviewing beat-gate PASS (no silent promote)" >&2
  exit 2
}
case "$LOOP" in
  config|ops|cognition|knowledge|discovery) ;;
  *) echo "error: unsupported loop '$LOOP'" >&2; usage ;;
esac
if [[ "${CUTOVER_APPROVED:-}" == "1" ]]; then
  echo "error: CUTOVER_APPROVED=1 is whole-host cutover — use cutover tooling, not promote-loop" >&2
  exit 2
fi
if [[ "$APPLY" == "1" ]]; then
  echo "error: PROMOTE_APPLY=1 not enabled yet — first ship is record-only (ADR-0005)" >&2
  exit 2
fi

mkdir -p "$OUT"
MUTEX_JSON="$("$ROOT/scripts/living-host-mutex.sh" status 2>/dev/null || echo '{}')"

echo "=== promote-loop: beat-gate fixture ($LOOP) ==="
META="$OUT/pre-${LOOP}-meta.json"
bash "$LAB/scripts/beat-gate.sh" --loop "$LOOP" --fixture --meta "$META"

echo "=== promote-loop: mutex / dual-writer ==="
export MUTEX_JSON OUT LOOP META ROOT
python3 - <<'PY'
import json, os, subprocess
from datetime import datetime, timezone
from pathlib import Path

out = Path(os.environ["OUT"])
loop = os.environ["LOOP"]
meta_path = Path(os.environ["META"])
root = Path(os.environ["ROOT"])
meta = json.loads(meta_path.read_text(encoding="utf-8"))
mutex = {}
try:
    mutex = json.loads(os.environ.get("MUTEX_JSON") or "{}")
except Exception:
    mutex = {}

gate = (meta.get("metrics") or {}).get("gate_passed")
beats = meta.get("beats_incumbent")
baseline_id = meta.get("baseline_id")
dual = mutex.get("dual_writer_risk")
claim = mutex.get("claim") or {}

ok = bool(gate) and bool(beats) and bool(baseline_id) and dual is not True
stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
payload = {
    "schema": "gzmo.promote_loop/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "ok": ok,
    "loop": loop,
    "mode": "record_only",
    "ack": True,
    "apply": False,
    "beats_incumbent": beats,
    "gate_passed": gate,
    "baseline_id": baseline_id,
    "baseline_path": meta.get("baseline_path"),
    "mutex": {
        "dual_writer_risk": dual,
        "claim_host": claim.get("host"),
        "claim_note": claim.get("note"),
    },
    "meta": str(meta_path),
    "advice": (
        "promote_loop_record_ok — review artifact; living apply not yet enabled"
        if ok
        else "promote_loop_blocked — need gate_passed+baseline_id and dual_writer_risk!=true"
    ),
    "next": [
        "Keep PROMOTE_APPLY off until living handoff recipe is reviewed",
        "bash scripts/living-host-mutex.sh claim|release around any living prove",
        "Whole-host still needs CUTOVER_APPROVED=1",
    ],
}
path = out / f"promote-{loop}-{stamp}.json"
path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
md = [
    f"# Promote-loop record — {loop}",
    "",
    f"Verdict: **{'OK' if ok else 'BLOCKED'}**",
    "",
    f"- gate_passed: `{gate}`",
    f"- baseline_id: `{baseline_id}`",
    f"- dual_writer_risk: `{dual}`",
    f"- mode: record_only (PROMOTE_APPLY refused)",
    "",
    payload["advice"],
    "",
]
(out / "latest.md").write_text("\n".join(md) + "\n", encoding="utf-8")
print(json.dumps({"ok": ok, "path": str(path), "advice": payload["advice"]}, indent=2))
raise SystemExit(0 if ok else 1)
PY
