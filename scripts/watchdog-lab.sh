#!/usr/bin/env bash
# Keep-lane lab: exercise missed-run watchdog (soft-fail YELLOW, never RED alone).
# Writes fixture job timestamps under data-next/scheduler-runs, runs
# `gzmo metabolism watchdog`, records proof under data-next/watchdog-lab/.
#
#   bash scripts/watchdog-lab.sh
#   GZMO_METABOLISM_STALE_SECS=60 bash scripts/watchdog-lab.sh   # force stale on old fixtures
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/watchdog-lab"
RUNS="${GZMO_SCHEDULER_RUNS:-$DATA/scheduler-runs}"
BIN="${GZMO_BIN:-${CARGO_TARGET_DIR:-$HOME/github-clone/temp-bench/target}/release/gzmo}"
export GZMO_INSTANCE="${GZMO_INSTANCE:-next}"
export GZMO_CONFIG="${GZMO_CONFIG:-$ROOT/config/gzmo-next.toml}"
export GZMO_ALLOW_LAB_VAULT="${GZMO_ALLOW_LAB_VAULT:-1}"
# Burst-friendly threshold so the lab proves STALE without waiting 26h.
export GZMO_METABOLISM_STALE_SECS="${GZMO_METABOLISM_STALE_SECS:-90}"

mkdir -p "$OUT" "$RUNS"

# Old finished times → stale under 90s threshold.
OLD="2020-01-01T00:00:00+00:00"
for job in distill dream; do
  cat >"$RUNS/latest-${job}.json" <<EOF
{
  "job": "${job}",
  "script": "watchdog-lab",
  "args": [],
  "started": "${OLD}",
  "finished": "${OLD}",
  "ok": true,
  "error": null,
  "runner": "lab"
}
EOF
done

PROBE_LOG="$OUT/watchdog.log"
PROBE_OK=0
rm -f "$OUT/watchdog.json"
if [[ -x "$BIN" ]]; then
  if ! "$BIN" metabolism help 2>&1 | rg -q 'watchdog'; then
    echo "binary lacks 'metabolism watchdog' — rebuild: cargo build --release -p gzmo-cli" >"$PROBE_LOG"
  elif "$BIN" metabolism watchdog >"$OUT/watchdog.json" 2>"$PROBE_LOG"; then
    PROBE_OK=1
  fi
else
  echo "no gzmo binary at $BIN" >"$PROBE_LOG"
fi

# Only trust JSON from this probe (do not reuse a prior serve-poll artifact).
SRC="$OUT/watchdog.json"
if [[ "$PROBE_OK" == "1" && -f "$RUNS/latest-watchdog.json" ]]; then
  # CLI also writes scheduler-runs/latest-watchdog.json — keep a copy for spine-demo.
  cp "$RUNS/latest-watchdog.json" "$OUT/latest-watchdog.json"
fi

export OUT RUNS SRC PROBE_OK PROBE_LOG BIN
python3 - <<'PY'
import json, os
from datetime import datetime, timezone
from pathlib import Path

out = Path(os.environ["OUT"])
src = Path(os.environ["SRC"])
wd = {}
if src.is_file() and src.stat().st_size > 0:
    wd = json.loads(src.read_text(encoding="utf-8"))

stale = bool(wd.get("stale"))
# Lab intent: with old fixtures + short threshold, stale must be true.
ok = os.environ["PROBE_OK"] == "1" and stale and "distill" in (wd.get("detail") or "")
payload = {
    "schema": "gzmo.watchdog.lab/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "ok": ok,
    "probe_ok": os.environ["PROBE_OK"] == "1",
    "stale": stale,
    "threshold_secs": wd.get("threshold_secs"),
    "detail": wd.get("detail"),
    "runs_dir": os.environ["RUNS"],
    "binary": os.environ["BIN"],
    "advice": (
        "watchdog_lab_pass — soft STALE detected under short threshold"
        if ok
        else "watchdog_lab_fail — rebuild gzmo or check metabolism watchdog path"
    ),
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
(out / "latest.md").write_text(
    f"# Watchdog lab\n\n"
    f"**ok:** {payload['ok']}\n\n"
    f"- stale: {payload['stale']}\n"
    f"- threshold_secs: {payload['threshold_secs']}\n"
    f"- detail: {payload['detail']}\n"
    f"- advice: {payload['advice']}\n",
    encoding="utf-8",
)
print(json.dumps({"ok": ok, "stale": stale, "advice": payload["advice"]}, indent=2))
raise SystemExit(0 if ok else 1)
PY
