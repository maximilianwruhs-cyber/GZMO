#!/usr/bin/env bash
# Unpark Wave 3 demable: Arena nightburst suggestion → Forge recommend → human pin ritual.
# Never writes living /opt/gzmo/gzmo.toml. Never starts gzmo-daemon Arena jobs.
#
#   bash scripts/arena-pin-demo.sh
#   bash scripts/arena-pin-demo.sh --skip-pin-log
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/arena-pin"
mkdir -p "$OUT"

SKIP_PIN_LOG=0
for arg in "$@"; do
  case "$arg" in
    --skip-pin-log) SKIP_PIN_LOG=1 ;;
    -h|--help)
      echo "Usage: $0 [--skip-pin-log]"
      exit 0
      ;;
  esac
done

echo "[1/5] arena-night-check (suggest-only artifact review)…"
bash "$ROOT/scripts/arena-night-check.sh" >/tmp/arena-pin-night-check.log 2>&1 || {
  echo "arena-night-check failed — see /tmp/arena-pin-night-check.log" >&2
  exit 1
}

echo "[2/5] arena-lab-demo (RAPL/€ observability; soft)…"
bash "$ROOT/scripts/arena-lab-demo.sh" >/tmp/arena-pin-lab-demo.log 2>&1 || true

echo "[3/5] forge-lab-demo (recommend pins; blocks_distill=false)…"
bash "$ROOT/scripts/forge-lab-demo.sh" >/tmp/arena-pin-forge-demo.log 2>&1 || {
  echo "forge-lab-demo failed — see /tmp/arena-pin-forge-demo.log" >&2
  exit 1
}

echo "[4/5] brain-intel-promote (human-only ritual)…"
bash "$ROOT/scripts/brain-intel-promote.sh" >/tmp/arena-pin-intel.log 2>&1 || {
  echo "brain-intel-promote failed — see /tmp/arena-pin-intel.log" >&2
  exit 1
}

if [[ "$SKIP_PIN_LOG" -eq 0 ]]; then
  echo "[5/5] pin-log theater samples (accept + reject; log only)…"
  # Ensure both decision flavors exist for Brain Feed pin-log readiness (never applies toml).
  NEED_ACCEPT=1
  NEED_REJECT=1
  PIN_ROLLUP=""
  for cand in "$DATA/brain-intel/pin-log-latest.json" "$DATA/brain-intel/pin-log.json"; do
    [[ -f "$cand" ]] && PIN_ROLLUP="$cand" && break
  done
  if [[ -n "$PIN_ROLLUP" ]]; then
    python3 -c "
import json
d=json.load(open('$PIN_ROLLUP'))
raise SystemExit(0 if d.get('accepted',0)>=1 else 1)
" 2>/dev/null && NEED_ACCEPT=0 || true
    python3 -c "
import json
d=json.load(open('$PIN_ROLLUP'))
raise SystemExit(0 if d.get('rejected',0)>=1 else 1)
" 2>/dev/null && NEED_REJECT=0 || true
  fi
  if [[ "$NEED_ACCEPT" -eq 1 ]]; then
    bash "$ROOT/scripts/brain-intel-pin-log.sh" \
      --decision accept \
      --reason "arena-pin theater: champion suggestion reviewed (log only; living toml unchanged)"
  fi
  if [[ "$NEED_REJECT" -eq 1 ]]; then
    bash "$ROOT/scripts/brain-intel-pin-log.sh" \
      --decision reject \
      --reason "arena-pin theater: estimate-only joules / not promoting tonight (log only)"
  fi
  # Always record a defer for this demo run (felt trail).
  bash "$ROOT/scripts/brain-intel-pin-log.sh" \
    --decision defer \
    --reason "arena-pin theater demo run — operator still decides living merge"
else
  echo "[5/5] pin-log skipped (--skip-pin-log)"
fi

export ROOT OUT DATA
python3 - <<'PY'
import json
import os
from datetime import datetime, timezone
from pathlib import Path

root = Path(os.environ["ROOT"])
out = Path(os.environ["OUT"])
data = Path(os.environ["DATA"])

champ = data / "arena" / "champion-suggestion.toml"
latest = data / "arena" / "latest.json"
forge_rec = data / "forge-lab" / "recommend.json"
intel = data / "brain-intel" / "latest.json"
pin_log = data / "brain-intel" / "pin-log-latest.json"
if not pin_log.is_file():
    pin_log = data / "brain-intel" / "pin-log.json"
pin_doc = data / "brain-intel" / "living-pin-suggestion.md"

champ_meta = {}
if champ.is_file():
    for line in champ.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if "=" not in line or line.startswith("#") or line.startswith("["):
            continue
        k, v = line.split("=", 1)
        champ_meta[k.strip()] = v.strip().strip('"')

night = None
if latest.is_file():
    try:
        night = json.loads(latest.read_text(encoding="utf-8"))
    except Exception as e:
        night = {"error": str(e)}

forge = None
if forge_rec.is_file():
    try:
        forge = json.loads(forge_rec.read_text(encoding="utf-8"))
    except Exception as e:
        forge = {"error": str(e)}

intel_p = None
if intel.is_file():
    try:
        intel_p = json.loads(intel.read_text(encoding="utf-8"))
    except Exception as e:
        intel_p = {"error": str(e)}

pins = None
if pin_log.is_file():
    try:
        pins = json.loads(pin_log.read_text(encoding="utf-8"))
    except Exception as e:
        pins = {"error": str(e)}

auto_apply = bool(night and night.get("auto_apply") is True)
daemon_touched = bool(night and night.get("daemon_jobs_touched") is True)
blocks = bool(forge and forge.get("blocks_distill") is True)

payload = {
    "schema": "gzmo.unpark.arena_pin.demo/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "wave": "3.pin",
    "ok": (
        champ.is_file()
        and not auto_apply
        and not daemon_touched
        and not blocks
        and bool(intel_p and intel_p.get("ok") is True)
    ),
    "auto_apply": False,
    "daemon_jobs_touched": False,
    "blocks_distill": False,
    "champion": {
        "path": str(champ) if champ.is_file() else None,
        "engine_label": champ_meta.get("engine_label"),
        "z": champ_meta.get("z"),
        "quality": champ_meta.get("quality"),
    },
    "nightburst": {
        "path": str(latest) if latest.is_file() else None,
        "z": (night or {}).get("z"),
        "quality": (night or {}).get("quality"),
        "energy_source": (night or {}).get("energy_source"),
        "euro_cost": (night or {}).get("euro_cost"),
    },
    "forge_recommend": {
        "path": str(forge_rec) if forge_rec.is_file() else None,
        "champion": (forge or {}).get("champion"),
        "pins": len((forge or {}).get("pins") or []),
        "blocks_distill": (forge or {}).get("blocks_distill"),
    },
    "brain_intel": {
        "path": str(intel) if intel.is_file() else None,
        "verdict": (intel_p or {}).get("verdict"),
        "pin_doc": str(pin_doc) if pin_doc.is_file() else None,
    },
    "pin_log": {
        "path": str(pin_log) if pin_log.is_file() else None,
        "accepted": (pins or {}).get("accepted"),
        "rejected": (pins or {}).get("rejected"),
        "deferred": (pins or {}).get("deferred"),
        "ok": (pins or {}).get("ok"),
    },
    "advice": (
        "arena_pin_demo_ok — suggest → recommend → human pin log; living toml untouched"
        if champ.is_file() and not auto_apply and not blocks
        else "arena_pin_demo_hold — missing champion or forbidden auto flags"
    ),
}
(out / "demo.json").write_text(json.dumps(payload, indent=2) + "\n")

lines = [
    "# Arena → Pin felt sample",
    "",
    f"Generated: {payload['generated_at']}",
    f"Verdict: {'OK' if payload['ok'] else 'HOLD'}",
    "",
    "Theater only — **not** Brain Feed GREEN claim, **not** living toml merge.",
    "",
    "## Champion suggestion (sibling)",
    "",
    f"- path: `{payload['champion']['path']}`",
    f"- engine: `{payload['champion']['engine_label']}`",
    f"- z / quality: `{payload['champion']['z']}` / `{payload['champion']['quality']}`",
    f"- energy_source: `{(payload['nightburst'] or {}).get('energy_source')}`",
    "",
    "## Forge recommend",
    "",
    f"- champion: `{payload['forge_recommend']['champion']}`",
    f"- pins: {payload['forge_recommend']['pins']}",
    f"- blocks_distill: **{payload['forge_recommend']['blocks_distill']}**",
    "",
    "## Human pin ritual",
    "",
    f"- brain-intel: `{payload['brain_intel']['verdict']}`",
    f"- pin doc: `{payload['brain_intel']['pin_doc']}`",
    f"- pin-log accept/reject/defer: "
    f"{payload['pin_log'].get('accepted')}/"
    f"{payload['pin_log'].get('rejected')}/"
    f"{payload['pin_log'].get('deferred')}",
    "",
    "## Hard rules",
    "",
    "1. Never auto-apply champion into `/opt/gzmo/gzmo.toml`.",
    "2. Never add Arena jobs to `gzmo-daemon` by default.",
    "3. Estimate joules ≠ RAPL trust until ACL/caps say so.",
    "",
]
(out / "felt-latest.md").write_text("\n".join(lines) + "\n", encoding="utf-8")
print(json.dumps({"ok": payload["ok"], "advice": payload["advice"], "champion": payload["champion"]}, indent=2))
PY

bash "$ROOT/scripts/arena-pin-check.sh"
echo "[OK] arena-pin demo → $OUT"
